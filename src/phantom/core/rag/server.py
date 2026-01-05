# src/server.py
"""
RAG server usando modelo LOCAL
Zero custo após setup inicial!
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from transformers import AutoTokenizer, AutoModelForCausalLM, BitsAndBytesConfig
import torch
from chromadb import PersistentClient
from typing import List, Optional
import json

app = FastAPI(title="CEREBRO RAG Server")

class QueryRequest(BaseModel):
    query: str
    top_k: int = 5
    include_context: bool = True

class QueryResponse(BaseModel):
    answer: str
    sources: List[dict]
    confidence: float

class CEREBROServer:
    def __init__(self, model_name: str, db_path: str, quantization: str = "4bit"):
        print(f"🧠 Loading CEREBRO with {model_name}...")

        # Load vector DB
        self.db = PersistentClient(path=db_path)
        self.collection = self.db.get_or_create_collection("cerebro_knowledge")

        # Load local model com quantização
        quantization_config = BitsAndBytesConfig(
            load_in_4bit=(quantization == "4bit"),
            bnb_4bit_compute_dtype=torch.float16,
            bnb_4bit_use_double_quant=True,
            bnb_4bit_quant_type="nf4"
        )

        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModelForCausalLM.from_pretrained(
            model_name,
            quantization_config=quantization_config,
            device_map="auto",
            trust_remote_code=True
        )

        print("✅ CEREBRO ready!")

    def retrieve(self, query: str, top_k: int = 5) -> List[dict]:
        """Retrieve relevant artifacts from knowledge base"""
        results = self.collection.query(
            query_texts=[query],
            n_results=top_k
        )

        # Parse metadata
        artifacts = []
        for i in range(len(results['ids'][0])):
            artifacts.append({
                'id': results['ids'][0][i],
                'content': results['documents'][0][i],
                'metadata': results['metadatas'][0][i],
                'distance': results['distances'][0][i] if 'distances' in results else None
            })

        return artifacts

    def generate(self, query: str, context: List[dict]) -> str:
        """Generate answer using local model + retrieved context"""

        # Build prompt com retrieved context
        context_text = "\n\n".join([
            f"Source: {c['metadata']['file_path']}\n{c['content']}"
            for c in context
        ])

        prompt = f"""You are a NixOS and development expert with deep knowledge of the user's codebases.

RETRIEVED CONTEXT:
{context_text}

USER QUESTION:
{query}

Provide a precise, technical answer based on the retrieved context. If the context doesn't contain enough information, say so clearly.

ANSWER:"""

        # Generate
        inputs = self.tokenizer(prompt, return_tensors="pt").to(self.model.device)

        with torch.no_grad():
            outputs = self.model.generate(
                **inputs,
                max_new_tokens=512,
                temperature=0.7,
                do_sample=True,
                top_p=0.95,
            )

        answer = self.tokenizer.decode(outputs[0][inputs['input_ids'].shape[1]:], skip_special_tokens=True)

        return answer.strip()

# Global instance
cerebro = None

@app.on_event("startup")
async def startup():
    global cerebro
    import os

    model_name = os.getenv("CEREBRO_MODEL", "TheBloke/Mistral-7B-Instruct-v0.2-GPTQ")
    db_path = os.getenv("CEREBRO_DB", "./data/cerebro.db")
    quantization = os.getenv("CEREBRO_QUANTIZATION", "4bit")

    cerebro = CEREBROServer(model_name, db_path, quantization)

@app.post("/query", response_model=QueryResponse)
async def query(request: QueryRequest):
    """Query the knowledge base"""
    if cerebro is None:
        raise HTTPException(500, "CEREBRO not initialized")

    # Retrieve relevant context
    sources = cerebro.retrieve(request.query, request.top_k)

    # Generate answer
    answer = cerebro.generate(request.query, sources)

    return QueryResponse(
        answer=answer,
        sources=sources,
        confidence=0.85  # TODO: implement proper confidence scoring
    )

@app.get("/health")
async def health():
    return {"status": "healthy", "model_loaded": cerebro is not None}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
