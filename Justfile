# Justfile - CEREBRO Automation

# Run test suite
test:
    poetry run pytest

# Show Cerebro environment info
info:
    poetry run cerebro info

# Run code analysis on a repository
analyze path context="General Review":
    poetry run cerebro knowledge analyze {{path}} "{{context}}"

# Sincroniza dados com o GCS (Staging para Ingestão)
sync local_dir="./data/analyzed":
    ./scripts/sync_data.sh {{local_dir}}

# Start RAG ingestion (Discovery Engine Import)
ingest:
    poetry run cerebro rag ingest

# Query the knowledge base
query question:
    poetry run cerebro rag query "{{question}}"

# Setup environment (Poetry)
install:
    poetry install

# Run full validation pipeline
pipeline:
    ./scripts/pipeline.sh
