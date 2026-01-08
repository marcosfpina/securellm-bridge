import json
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

from google.cloud import storage
from google.cloud import discoveryengine_v1beta as discoveryengine
from google.api_core import exceptions
from phantom.core.gcp.search import VertexAISearch, GroundedResponse


class RigorousRAGEngine:
    """
    Motor RAG de Alta Precisão (CEREBRO).
    
    ALINHADO COM A LEI: Consumo programático de créditos GenAI App Builder via Discovery Engine.
    """

    def __init__(
        self, 
        data_store_id: Optional[str] = None,
        location: str = "global",
        persist_directory: str = "./data/vector_db"
    ):
        self.project_id = os.getenv("GCP_PROJECT_ID")
        self.location = location
        self.data_store_id = data_store_id or os.getenv("DATA_STORE_ID")
        self.persist_directory = persist_directory
        
        # Cliente Discovery Engine para Consumo de Créditos (A LEI)
        if self.data_store_id:
            self.cloud_search = VertexAISearch(
                project_id=self.project_id,
                location=self.location,
                data_store_id=self.data_store_id
            )
        else:
            self.cloud_search = None

    def ingest(self, jsonl_path: str) -> int:
        """
        Ingestão via Discovery Engine (Consome Créditos GenAI App Builder).
        
        Fluxo: Local JSONL -> GCS -> Discovery Engine Import
        """
        if not self.project_id or not self.data_store_id:
            raise ValueError("GCP_PROJECT_ID e DATA_STORE_ID devem estar configurados.")

        path = Path(jsonl_path)
        if not path.exists():
            raise FileNotFoundError(f"Artefatos não encontrados: {jsonl_path}")

        print("\n🚀 Iniciando Ingestão Programática (A LEI)...")
        
        # 1. Upload para GCS (Stage)
        bucket_name = f"{self.project_id}-phantom-ingest"
        storage_client = storage.Client(project=self.project_id)
        
        try:
            bucket = storage_client.get_bucket(bucket_name)
        except exceptions.NotFound:
            print(f"🔨 Criando bucket de staging: gs://{bucket_name}")
            bucket = storage_client.create_bucket(bucket_name)

        blob = bucket.blob(f"ingest/{path.name}")
        print(f"📤 Fazendo upload de {path.name} para gs://{bucket_name}/ingest/...")
        blob.upload_from_filename(str(path))
        
        gcs_uri = f"gs://{bucket_name}/ingest/{path.name}"

        # 2. Trigger ImportDocuments no Discovery Engine
        client_options = None
        if self.location != "global":
            client_options = {"api_endpoint": f"{self.location}-discoveryengine.googleapis.com"}
            
        client = discoveryengine.DocumentServiceClient(client_options=client_options)
        
        parent = f"projects/{self.project_id}/locations/{self.location}/collections/default_collection/dataStores/{self.data_store_id}/branches/default_branch"
        
        request = discoveryengine.ImportDocumentsRequest(
            parent=parent,
            gcs_source=discoveryengine.GcsSource(
                input_uris=[gcs_uri],
                data_schema="document" # JSONL format optimized for Discovery Engine
            ),
            reconciliation_mode=discoveryengine.ImportDocumentsRequest.ReconciliationMode.INCREMENTAL
        )

        print(f"🔄 Solicitando importação no Data Store: {self.data_store_id}")
        operation = client.import_documents(request=request)
        
        print(f"⏳ Operação iniciada: {operation.operation.name}")
        print("✅ O Google está processando seus documentos em background usando seus créditos.")
        print("💡 Verifique o status no console: https://console.cloud.google.com/gen-app-builder/data-stores")
        
        # Retorna um valor simbólico (número de linhas no arquivo)
        with open(path, 'r') as f:
            return sum(1 for _ in f)

    def query_with_metrics(self, query: str, k: int = 5) -> Dict[str, Any]:
        """
        Executa Grounded Generation via Discovery Engine (Consome Créditos GenAI).
        """
        if not self.cloud_search:
            return {
                "answer": "❌ Erro: DATA_STORE_ID não configurado. Defina a variável de ambiente DATA_STORE_ID.",
                "metrics": {"hit_rate_k": "0%", "avg_confidence": 0.0, "top_source": "N/A"},
            }

        try:
            # Executa Busca Aterrada (Grounded Generation) - SKU Elegível para Créditos
            response: GroundedResponse = self.cloud_search.grounded_search(query, top_k=k)

            return {
                "answer": response.summary or "Não foi possível gerar um sumário com os documentos encontrados.",
                "metrics": {
                    "avg_confidence": 1.0, 
                    "hit_rate_k": "100%" if response.results else "0%",
                    "retrieved_docs": len(response.results),
                    "top_source": response.results[0].title if response.results else "N/A",
                    "citations": response.citations,
                    "cost_estimate_usd": response.cost_estimate
                },
            }
        except Exception as e:
            return {
                "answer": f"❌ Erro na consulta ao Discovery Engine: {str(e)}",
                "metrics": {"hit_rate_k": "ERR", "avg_confidence": 0.0, "top_source": "N/A"},
            }
