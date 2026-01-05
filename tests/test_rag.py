
import json
import pytest
from unittest.mock import MagicMock, patch, mock_open
from phantom.core.rag.engine import RigorousRAGEngine

@pytest.fixture
def mock_vertex_embeddings():
    with patch("phantom.core.rag.engine.VertexAIEmbeddings") as mock:
        yield mock

@pytest.fixture
def mock_vertex_ai():
    with patch("phantom.core.rag.engine.VertexAI") as mock:
        yield mock

@pytest.fixture
def mock_chroma():
    with patch("phantom.core.rag.engine.Chroma") as mock:
        yield mock

def test_initialization(mock_vertex_embeddings, mock_vertex_ai):
    engine = RigorousRAGEngine(persist_directory="./test_db")
    assert engine.persist_directory == "./test_db"
    mock_vertex_embeddings.assert_called_once()
    mock_vertex_ai.assert_called_once()

def test_ingest_file_not_found(mock_vertex_embeddings, mock_vertex_ai):
    engine = RigorousRAGEngine()
    with pytest.raises(FileNotFoundError):
        engine.ingest("non_existent_file.jsonl")

@patch("builtins.open", new_callable=mock_open, read_data='{"jsonData": "{\\"title\\": \\"test\\", \\"content\\": \\"code content\\", \\"repo\\": \\"repo1\\"}"}\n')
@patch("phantom.core.rag.engine.Path.exists", return_value=True)
def test_ingest_success(mock_exists, mock_file, mock_vertex_embeddings, mock_vertex_ai, mock_chroma):
    engine = RigorousRAGEngine()
    
    # Mock Chroma.from_documents
    mock_db = MagicMock()
    mock_chroma.from_documents.return_value = mock_db
    
    count = engine.ingest("test.jsonl")
    
    assert count > 0
    mock_chroma.from_documents.assert_called_once()
    mock_db.persist.assert_called_once()

def test_query_with_metrics_no_context(mock_vertex_embeddings, mock_vertex_ai, mock_chroma):
    engine = RigorousRAGEngine()
    
    # Mock Vector DB retrieval returning empty
    mock_db = MagicMock()
    mock_db.similarity_search_with_relevance_scores.return_value = []
    mock_chroma.return_value = mock_db
    
    result = engine.query_with_metrics("test query")
    
    assert result["answer"] == "No context found."
    assert result["metrics"]["hit_rate"] == 0.0

def test_query_with_metrics_success(mock_vertex_embeddings, mock_vertex_ai, mock_chroma):
    engine = RigorousRAGEngine()
    
    # Mock Vector DB retrieval
    mock_doc = MagicMock()
    mock_doc.page_content = "context content"
    mock_doc.metadata = {"source": "repo/file"}
    mock_db = MagicMock()
    mock_db.similarity_search_with_relevance_scores.return_value = [(mock_doc, 0.8), (mock_doc, 0.6)]
    mock_chroma.return_value = mock_db
    
    # Mock LLM response
    mock_chain = MagicMock()
    mock_chain.invoke.return_value = "AI Answer"
    
    # We need to mock the chain creation. 
    # The code does: chain = prompt | self.llm
    # We can mock the prompt template or just the llm invoke if it was direct, but it uses a chain.
    # A simpler way is to mock invoke on the result of the OR operation.
    # However, since PromptTemplate | LLM returns a Runnable, we can mock the LLM's __ror__ (reverse or) or similar?
    # Actually, in LangChain, `prompt | llm` works. 
    # Let's mock the `invoke` on the chain. 
    
    with patch("phantom.core.rag.engine.PromptTemplate") as MockPrompt:
        mock_prompt_instance = MockPrompt.return_value
        # When prompt | llm happens
        mock_runnable = MagicMock()
        mock_runnable.invoke.return_value = "AI Answer"
        mock_prompt_instance.__or__.return_value = mock_runnable
        
        result = engine.query_with_metrics("test query")
        
        assert result["answer"] == "AI Answer"
        assert result["metrics"]["avg_confidence"] == 0.7
        assert "25%" in result["metrics"]["hit_rate_k"] # 1 out of 4 (default k) is > 0.7
