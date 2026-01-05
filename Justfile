# Justfile - PHANTOM Automation

# Run test suite
test:
    poetry run pytest

# Show Phantom environment info
info:
    poetry run phantom info

# Run code analysis on a repository
analyze path context="General Review":
    poetry run phantom knowledge analyze {{path}} "{{context}}"

# Start RAG ingestion (requires GCP/LangChain)
ingest:
    poetry run phantom rag ingest

# Query the knowledge base
query question:
    poetry run phantom rag query "{{question}}"

# Setup environment (Poetry)
install:
    poetry install

# Run full validation pipeline
pipeline:
    ./scripts/pipeline.sh
