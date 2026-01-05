# 🧠 PHANTOM Framework

**Knowledge Extraction & Cloud Credit Management**

Version 2.0 - Unified framework for code intelligence and GCP credit consumption

---

## 🔥 NEW: ROI Strategy Documentation

**Goal:** Transform R$ 10k in GCP credits → R$ 500k+ in career value

### 📚 Complete Documentation Suite

| Document | Purpose | Start Here |
|----------|---------|------------|
| **[INDEX.md](INDEX.md)** | Navigation hub | ⭐ Overview |
| **[STACK_MASTERY.md](STACK_MASTERY.md)** | Complete stack guide | ⭐⭐⭐ Learn |
| **[EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md)** | Complete roadmap & strategy | ⭐⭐⭐ Read first |
| **[HACKS_ROI.md](HACKS_ROI.md)** | High-ROI hacks & strategies | ⭐⭐⭐ Strategy |
| **[HIGH_ROI_QUERIES.md](HIGH_ROI_QUERIES.md)** | 189 ready-to-use queries | ⭐⭐⭐ Execute |
| **[AUTOMATION_SYSTEMS.md](AUTOMATION_SYSTEMS.md)** | Long-term automation systems | Scale |
| **[README_SPEEDRUN.md](README_SPEEDRUN.md)** | Detailed execution guide | Reference |
| **[QUICKSTART_KB.md](QUICKSTART_KB.md)** | Technical deep dive | Technical |
| **[CHEATSHEET.md](CHEATSHEET.md)** | Quick reference | Daily use |

### ⚡ Quick Start (3 comandos)

```bash
export ENGINE_ID=seu-discovery-engine-id
./speedrun.sh setup      # Valida ambiente
./speedrun.sh all        # Gera queries + queima créditos
```

**Read:** [EXECUTIVE_SUMMARY.md](EXECUTIVE_SUMMARY.md) for complete roadmap

### 🔥 NEW: Arsenal de Scripts (Weaponized Intelligence)

**10 scripts de alto ROI criados:**

| Script | ROI | Quando Usar |
|--------|-----|-------------|
| **strategy_optimizer.py** | Meta | PRIMEIRO - te diz o que fazer |
| **salary_intel.py** | 2000x | Negotiation → R$ 50k-200k |
| **content_gold_miner.py** | ∞ | Mine outputs → Viral content |
| **trend_predictor.py** | 50x | Early mover advantage |
| **personal_moat_builder.py** | 200x | Niche expertise única |
| **generate_queries.py** | 10x | Volume/broad knowledge |
| **batch_burn.py** | - | Execute queries |
| **monitor_credits.py** | - | Track spending |
| **create_search_engine.py** | - | Setup: Create Discovery Engine |
| **index_repository.py** | - | Setup: Index repo content |

**Start here:** `python scripts/strategy_optimizer.py`

**Full docs:** [scripts/README_ARSENAL.md](scripts/README_ARSENAL.md)

---

## 🎯 What is Phantom?

Phantom is a unified framework that combines:

1. **Knowledge Extraction & RAG** - Analyze codebases and create queryable knowledge bases using local LLMs
2. **GCP Credit Management** - Programmatically consume Google Cloud promotional credits with complete auditing
3. **Multi-Cloud Integrations** - Extensible architecture for cloud service integrations

## 🚀 Quick Start

### Installation

```bash
# Clone and enter directory
cd phoenix-cloud-run

# Enter Nix development environment
nix develop

# Verify installation
python phantom.py info
```

### GCP Setup

```bash
# 1. Authenticate
gcloud auth application-default login

# 2. Validate setup
python phantom.py gcp validate

# 3. Create a data store
python phantom.py gcp datastores-create my-datastore

# 4. Set environment
export DATA_STORE_ID='my-datastore'

# 5. Test search
python phantom.py gcp search "What is machine learning?"
```

### Burn Credits

```bash
# GenAI App Builder credits
python phantom.py credit loadtest --num-queries 100 --workers 10

# Dialogflow CX credits
export DIALOGFLOW_AGENT_ID='your-agent-id'
python phantom.py credit dialogflow --num-conversations 50

# Audit consumption
python phantom.py credit audit --days 7
```

## 📚 Documentation

- **[Architecture](phantom/docs/ARCHITECTURE.md)** - Complete framework architecture
- **[Quick Start](phantom/docs/QUICK_START.md)** - Fast setup guide
- **[Credit Burner Details](phantom/docs/CREDIT_BURNER_DETAILED.md)** - Comprehensive credit burning guide
- **[Executive Summary](phantom/docs/RESUMO_EXECUTIVO.md)** - Project overview (Portuguese)

## 🏗️ Project Structure

```
.
├── phantom.py                 # Main CLI entry point
├── phantom/                   # Framework directory
│   ├── core/                 # Core modules
│   │   ├── gcp/             # Google Cloud Platform
│   │   ├── extraction/      # Code analysis
│   │   ├── rag/             # RAG server
│   │   └── utils/           # Utilities
│   ├── modules/             # Application modules
│   │   ├── credit_burner/  # Credit consumption
│   │   ├── knowledge/      # Knowledge management
│   │   └── nixos/          # NixOS integration
│   ├── config/             # Configuration files
│   ├── docs/               # Documentation
│   ├── scripts/            # Utility scripts
│   └── tests/              # Test suite
├── flake.nix              # Nix environment
└── requirements.txt       # Python dependencies
```

## 🎨 Key Features

### Core/GCP Module

Unified Google Cloud Platform integrations:

- ✅ **Authentication** - Simplified GCP auth validation
- ✅ **Data Stores** - Vertex AI Data Store management
- ✅ **Search** - Grounded generation with citations
- ✅ **Billing** - BigQuery-based credit auditing
- ✅ **Dialogflow CX** - Conversation session management

### Credit Burner Module

Programmatic credit consumption with validation:

- **GenAI App Builder** - R$ 6,432.54 via Vertex AI Search
- **Dialogflow CX** - R$ 3,646.57 via conversation sessions
- **BigQuery Audit** - Real-time financial validation
- **Parallel Execution** - Multi-threaded load testing

### Extraction Module

Code intelligence and knowledge extraction:

- **Multi-Language** - Python, Nix, Rust, Go support
- **AST Analysis** - Deep code structure understanding
- **Tree-sitter** - Fast, accurate parsing
- **Embeddings** - Vector representations for search

### RAG Module

Retrieval-Augmented Generation server:

- **Local LLM** - Mistral-7B with 4-bit quantization
- **ChromaDB** - Vector database for fast retrieval
- **FastAPI** - REST API server
- **NixOS Service** - SystemD integration

## 🛠️ CLI Commands

```bash
# GCP Operations
phantom gcp validate                    # Validate GCP setup
phantom gcp datastores-list             # List data stores
phantom gcp datastores-create <id>      # Create data store
phantom gcp search "query"              # Grounded search

# Credit Management
phantom credit loadtest                 # Burn GenAI credits
phantom credit dialogflow               # Burn Dialogflow credits
phantom credit audit                    # Audit via BigQuery

# RAG Operations
phantom rag serve                       # Start RAG server (coming soon)

# Information
phantom info                            # Framework info
phantom version                         # Version info
```

## 💡 Use Cases

### 1. Credit Consumption

Consume R$ 10,079.11 in GCP promotional credits through validated APIs:

```bash
# Setup
export DATA_STORE_ID='my-datastore'

# Burn credits (parallelized)
phantom credit loadtest --num-queries 1600 --workers 10

# Validate (24-48h later)
phantom credit audit --days 7
```

### 2. Knowledge Base

Create a queryable knowledge base from your codebases:

```bash
# Extract code (inside phantom/)
cd phantom
cerebro-extract

# Start RAG server
cerebro-serve

# Query knowledge
cerebro query "How does authentication work?"
```

### 3. Cloud Search

Use Vertex AI Search with grounded generation:

```bash
# Single query
phantom gcp search "What is RAG?"

# Get citations and sources
phantom gcp search "Explain NixOS" --top-k 10
```

## 🔧 Configuration

### Environment Variables

```bash
# GCP
export GOOGLE_CLOUD_PROJECT='your-project-id'
export GOOGLE_CLOUD_LOCATION='global'
export DATA_STORE_ID='your-datastore-id'

# Dialogflow
export DIALOGFLOW_AGENT_ID='agent-uuid'
export DIALOGFLOW_LOCATION='us-central1'

# BigQuery Billing
export BILLING_EXPORT_DATASET='billing_export'
export BILLING_EXPORT_TABLE='gcp_billing_export_v1_...'

# RAG Server
export CEREBRO_MODEL='TheBloke/Mistral-7B-Instruct-v0.2-GPTQ'
export CEREBRO_DB='./data/vector_db'
```

### Credits Available

| Credit | Value | Valid Until | Module |
|--------|-------|-------------|--------|
| GenAI App Builder Trial | R$ 6,432.54 | 2026-05-21 | Vertex AI Search |
| Dialogflow CX Trial | R$ 3,646.57 | 2026-11-30 | Dialogflow CX |

## 📊 Pricing

### Vertex AI Search
- **Search Enterprise**: $4.00 / 1,000 queries
- **Grounded Generation**: Included
- **With R$ 6,432.54**: ~1,608 queries

### Dialogflow CX
- **Text Session**: ~$0.007 per interaction
- **With R$ 3,646.57**: ~93,500 interactions

## 🧪 Development

```bash
# Enter development environment
nix develop

# Run tests (when available)
pytest tests/

# Type checking
mypy phantom/

# Linting
ruff check phantom/
```

## 🏛️ Architecture Principles

- **DRY**: No code duplication - single source of truth
- **Modular**: Independent, reusable components
- **Extensible**: Easy to add new modules/integrations
- **Hermetic**: Isolated, testable operations
- **Nix-First**: Reproducible environments

## 🔗 Links

- **Vertex AI Search**: https://cloud.google.com/generative-ai-app-builder
- **Dialogflow CX**: https://cloud.google.com/dialogflow/cx
- **BigQuery**: https://cloud.google.com/bigquery
- **NixOS**: https://nixos.org

## 📝 Migration Notes

This project was migrated from `phoenix-cloud-run` to the unified **Phantom Framework** architecture. Key changes:

- `cerebro/` → `phantom/` (unified framework)
- Duplicated files consolidated into `core/gcp/`
- Application logic moved to `modules/`
- Single CLI entry point: `phantom.py`

See `phantom/docs/ARCHITECTURE.md` for complete migration details.

## 🤝 Contributing

This is a personal project for consuming GCP promotional credits and building a knowledge extraction system. Not currently accepting external contributions.

## 📜 License

Private project - All rights reserved

---

**Framework Version**: 2.0.0
**Last Updated**: 2026-01-02
