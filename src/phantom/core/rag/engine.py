import json
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

from google.cloud import storage
from google.api_core import exceptions

from phantom.interfaces.llm import LLMProvider
from phantom.interfaces.vector_store import VectorStoreProvider
from phantom.providers.gcp.vertex_ai_llm import VertexAILLMProvider
from phantom.providers.chroma.chroma_vector_store import ChromaVectorStoreProvider


class RigorousRAGEngine:
    """
    Motor RAG de Alta Precisão (CEREBRO).
    
    ALINHADO COM A LEI: Consumo programático de créditos GenAI App Builder via Discovery Engine.
    
    Now uses dependency injection with pluggable LLM and Vector Store providers.
    """

    def __init__(
        self,
        llm_provider: Optional[LLMProvider] = None,
        vector_store_provider: Optional[VectorStoreProvider] = None,
        data_store_id: Optional[str] = None,
        location: str = "global",
        persist_directory: str = "./data/vector_db"
    ):
        """
        Initialize RigorousRAGEngine with pluggable providers.
        
        Args:
            llm_provider: LLMProvider instance (defaults to VertexAILLMProvider)
            vector_store_provider: VectorStoreProvider instance (defaults to ChromaVectorStoreProvider)
            data_store_id: Discovery Engine data store ID
            location: GCP location
            persist_directory: Directory for vector store persistence
        """
        self.project_id = os.getenv("GCP_PROJECT_ID")
        self.location = location
        self.data_store_id = data_store_id or os.getenv("DATA_STORE_ID")
        self.persist_directory = persist_directory
        
        # Initialize providers with defaults if not provided
        if llm_provider is None:
            self.llm_provider = VertexAILLMProvider(
                project_id=self.project_id,
                location=self.location,
                data_store_id=self.data_store_id
            )
        else:
            self.llm_provider = llm_provider

        if vector_store_provider is None:
            self.vector_store_provider = ChromaVectorStoreProvider(
                persist_directory=persist_directory
            )
        else:
            self.vector_store_provider = vector_store_provider

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

        # 2. Use LLM provider to import documents
        print(f"🔄 Solicitando importação no Data Store: {self.data_store_id}")
        operation_name = self.llm_provider.import_documents(gcs_uri)
        
        print(f"⏳ Operação iniciada: {operation_name}")
        print("✅ O Google está processando seus documentos em background usando seus créditos.")
        print("💡 Verifique o status no console: https://console.cloud.google.com/gen-app-builder/data-stores")
        
        # Retorna um valor simbólico (número de linhas no arquivo)
        with open(path, 'r') as f:
            return sum(1 for _ in f)

    def query_with_metrics(self, query: str, k: int = 5) -> Dict[str, Any]:
        """
        Executa Grounded Generation via Discovery Engine (Consome Créditos GenAI).
        """
        try:
            # Use LLM provider for grounded generation
            result = self.llm_provider.grounded_generate(
                query=query,
                context=[],  # Context comes from data store
                top_k=k
            )

            return {
                "answer": result.get("answer", "Não foi possível gerar um sumário com os documentos encontrados."),
                "metrics": {
                    "avg_confidence": result.get("confidence", 0.0),
                    "hit_rate_k": "100%" if result.get("citations") else "0%",
                    "retrieved_docs": len(result.get("citations", [])),
                    "top_source": result.get("citations", ["N/A"])[0] if result.get("citations") else "N/A",
                    "citations": result.get("citations", []),
                    "cost_estimate_usd": result.get("cost_estimate", 0.0)
                },
            }
        except Exception as e:
            return {
                "answer": f"❌ Erro na consulta ao Discovery Engine: {str(e)}",
                "metrics": {"hit_rate_k": "ERR", "avg_confidence": 0.0, "top_source": "N/A"},
            }
