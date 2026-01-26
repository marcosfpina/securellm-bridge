# Dockerfile
# Multi-stage build for Cerebro

# Stage 1: Builder
FROM python:3.13-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Poetry
RUN curl -sSL https://install.python-poetry.org | python3 -
ENV PATH="/root/.local/bin:$PATH"

# Copy project files
COPY pyproject.toml poetry.lock ./
COPY src/ ./src/

# Install dependencies
RUN poetry config virtualenvs.in-project true && \
    poetry install --no-dev

# Stage 2: Runtime
FROM python:3.13-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy virtual environment from builder
COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/src /app/src
COPY --from=builder /app/pyproject.toml /app/

# Set environment variables
ENV PATH="/app/.venv/bin:$PATH" \
    PYTHONUNBUFFERED=1 \
    PYTHONPATH="/app/src:$PYTHONPATH"

# Create data directories
RUN mkdir -p /app/data/analyzed /app/data/vector_db /app/data/reports

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cerebro ops health || exit 1

# Default command
CMD ["cerebro", "--help"]

# For Cloud Run, expose port 8000
EXPOSE 8000

# Alternative: Run RAG server
# CMD ["python", "-m", "phantom.core.rag.server"]
