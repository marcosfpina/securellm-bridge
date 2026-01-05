#!/usr/bin/env python3
"""
03_ingest_data.py - O JEITO CERTO DE INGERIR (Burn Credits Mode)
Em vez de gerar embeddings na mão (caro), importamos para o Data Store.
O Vertex AI Search faz a indexação/vetorização por conta da casa (créditos).
"""
import os
import json
from typing import List, Dict
from google.cloud import discoveryengine_v1beta as discoveryengine
from google.api_core.client_options import ClientOptions

def import_documents(project_id: str, location: str, data_store_id: str, input_jsonl: str):
    print(f"🚀 Iniciando Ingestão Segura (Credit-Funded)...")

    # 1. Configura Client no Endpoint Discovery Engine (Créditos ✅)
    client_options = (
        ClientOptions(api_endpoint=f"{location}-discoveryengine.googleapis.com")
        if location != "global" else None
    )
    client = discoveryengine.DocumentServiceClient(client_options=client_options)

    # 2. Define o caminho do Data Store
    parent = client.branch_path(
        project=project_id,
        location=location,
        data_store=data_store_id,
        branch="default_branch"
    )

    # 3. Prepara o Request de Importação (GCS ou Inline)
    # Para 2600 arquivos, o ideal é subir o JSONL para o GCS (Google Cloud Storage) primeiro
    # Mas aqui vamos assumir que você tem um JSONL local e vamos usar Inline (limite menor) ou GCS.
    # Recomendação: Use GCS para "Burn Credits" em massa.

    print("⚠️  Para ingestão em massa, suba o arquivo 'all_artifacts.jsonl' para um Bucket GCS.")
    gcs_uri = f"gs://{project_id}-staging/all_artifacts.jsonl"

    request = discoveryengine.ImportDocumentsRequest(
        parent=parent,
        gcs_source=discoveryengine.GcsSource(input_uris=[gcs_uri]),
        # O modo INCREMENTAL garante que você possa rodar de novo sem quebrar tudo
        reconciliation_mode=discoveryengine.ImportDocumentsRequest.ReconciliationMode.INCREMENTAL,
    )

    # 4. Dispara a operação (Long Running)
    operation = client.import_documents(request=request)
    print(f"⏳ Importação iniciada: {operation.operation.name}")
    print("   O Vertex AI Search agora está gerando embeddings e indexando (coberto pelo crédito).")

    # Opcional: operation.result() para esperar

if __name__ == "__main__":
    # Exemplo de uso
    import_documents(
        project_id=os.getenv("GCP_PROJECT"),
        location="global",
        data_store_id=os.getenv("DATA_STORE_ID"),
        input_jsonl="data/analyzed/all_artifacts.jsonl"
    )
