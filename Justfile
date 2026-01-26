# Justfile - CEREBRO Automation

# ============================================================================
# TESTING
# ============================================================================

# Run all tests
test:
    poetry run pytest

# Run unit tests only
test-unit:
    poetry run pytest tests/ -v --ignore=tests/integration --cov=src/phantom --cov-report=term

# Run integration tests only
test-integration:
    poetry run pytest tests/integration/ -v -m integration

# Run Vertex AI limit tests
test-vertex-limits:
    poetry run pytest tests/integration/test_vertex_limits.py -v -m integration

# ============================================================================
# CODE QUALITY
# ============================================================================

# Run linting with ruff
lint:
    poetry run ruff check src/ tests/

# Run linting and fix issues
lint-fix:
    poetry run ruff check --fix src/ tests/

# Format code with ruff
format:
    poetry run ruff format src/ tests/

# Type checking with mypy
type-check:
    poetry run mypy src/phantom --ignore-missing-imports

# Run all quality checks (lint + format + tests)
quality: lint format type-check test

# ============================================================================
# CI/CD SPECIFIC
# ============================================================================

# Run CI pipeline locally (simulates GitLab CI)
ci-local:
    @echo "Running local CI pipeline..."
    just validate-imports
    just validate-syntax
    just lint
    just format
    just test-unit
    @echo "✅ Local CI pipeline passed!"

# Validate imports
validate-imports:
    poetry run python -c "from phantom.core import gcp"
    poetry run python -c "from phantom.modules import credit_burner"
    poetry run python -c "import typer; import rich"

# Validate syntax
validate-syntax:
    find src/phantom/ -name "*.py" -exec python -m py_compile {} \;

# Run CLI tests
test-cli:
    poetry run cerebro --help
    poetry run cerebro info
    poetry run cerebro version
    poetry run cerebro ops status

# ============================================================================
# DOCKER
# ============================================================================

# Build Docker image
docker-build:
    docker build -t cerebro:latest .

# Run Docker image
docker-run:
    docker run -it --rm cerebro:latest cerebro --help

# ============================================================================
# DEPLOYMENT
# ============================================================================

# Deploy to Cloud Run (requires GCP credentials)
deploy-cloud-run:
    gcloud run deploy cerebro-api \
        --source . \
        --region us-central1 \
        --platform managed \
        --allow-unauthenticated \
        --set-env-vars "GCP_PROJECT_ID=gen-lang-client-0530325234"

# ============================================================================
# UTILITIES
# ============================================================================

# Show Cerebro environment info
info:
    poetry run cerebro info

# Run health check
health:
    poetry run cerebro ops health

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

# Clean build artifacts
clean:
    rm -rf build/ dist/ *.egg-info .pytest_cache .coverage htmlcov
    find . -type d -name __pycache__ -exec rm -rf {} +
    find . -type f -name "*.pyc" -delete
