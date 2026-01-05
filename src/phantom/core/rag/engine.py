import json
from pathlib import Path
from typing import Any, Dict

from langchain_community.vectorstores import Chroma
from langchain_core.documents import Document
from langchain_core.prompts import PromptTemplate
from langchain_google_vertexai import VertexAI, VertexAIEmbeddings
from langchain_text_splitters import RecursiveCharacterTextSplitter


class RigorousRAGEngine:
    def __init__(self, persist_directory: str = "./data/vector_db"):
        self.persist_directory = persist_directory
        self.embeddings = VertexAIEmbeddings(model="textembedding-gecko@003")
        self.llm = VertexAI(model="gemini-pro", temperature=0.0)
        self.vector_db = None

    def ingest(self, jsonl_path: str) -> int:
        """
        Ingere artefatos com controle estrito de metadados.
        """
        path = Path(jsonl_path)
        if not path.exists():
            raise FileNotFoundError(f"Artifacts not found: {jsonl_path}")

        # Carregamento Customizado para Preservar Metadados
        documents = []
        with open(path, "r") as f:
            for line in f:
                data = json.loads(line)
                inner = json.loads(data["jsonData"])
                doc = Document(
                    page_content=f"TITLE: {inner['title']}\nCONTENT:\n{inner['content']}",
                    metadata={
                        "source": f"{inner.get('repo', 'unknown')}/{inner.get('title', 'untitled')}",
                        "repo": inner.get("repo", ""),
                        "type": "code_artifact",
                        "context": inner.get("context", "N/A"),
                    },
                )
                documents.append(doc)

        # Splitting Otimizado para Código
        splitter = RecursiveCharacterTextSplitter(
            chunk_size=2000,
            chunk_overlap=200,
            separators=["\nclass ", "\ndef ", "\n\n", "\n", " ", ""],
        )
        texts = splitter.split_documents(documents)

        # Indexação Local (Chroma)
        self.vector_db = Chroma.from_documents(
            documents=texts,
            embedding=self.embeddings,
            persist_directory=self.persist_directory,
        )
        self.vector_db.persist()
        return len(texts)

    def query_with_metrics(self, query: str, k: int = 4) -> Dict[str, Any]:
        """
        Executa retrieval e retorna métricas de precisão (Hit Rate Proxy).
        """
        if not self.vector_db:
            self.vector_db = Chroma(
                persist_directory=self.persist_directory,
                embedding_function=self.embeddings,
            )

        # 1. Retrieval com Scores
        docs_with_scores = self.vector_db.similarity_search_with_relevance_scores(
            query, k=k
        )

        if not docs_with_scores:
            return {
                "answer": "No context found.",
                "metrics": {"hit_rate": 0.0, "avg_score": 0.0},
            }

        # 2. Cálculo de Métricas
        scores = [score for _, score in docs_with_scores]
        avg_score = sum(scores) / len(scores) if scores else 0.0
        hit_rate = (
            len([s for s in scores if s > 0.7]) / k
        )  # Exemplo: Score > 0.7 é "Hit"

        # 3. Geração Aterrada
        context_text = "\n\n".join([d.page_content for d, _ in docs_with_scores])
        prompt = PromptTemplate(
            template="""Você é um Arquiteto de Software Sênior (PHANTOM).
Responda a pergunta baseada ESTRITAMENTE no contexto abaixo. Seja técnico e direto.

CONTEXTO:
{context}

PERGUNTA:
{question}

RESPOSTA (Markdown):""",
            input_variables=["context", "question"],
        )

        chain = prompt | self.llm
        response = chain.invoke({"context": context_text, "question": query})

        return {
            "answer": response,
            "metrics": {
                "avg_confidence": round(avg_score, 4),
                "hit_rate_k": f"{hit_rate:.0%}",
                "retrieved_docs": k,
                "top_source": docs_with_scores[0][0].metadata.get("source", "unknown"),
            },
        }
