---

CLAUDE.md para Phoenix Cloud Run / PHANTOM Framework

# PHANTOM Framework v2.0 - Development Instructions

## Project Context

**PHANTOM** = Unified framework for GCP credit consumption + RAG/knowledge extraction

- **Location**: `/home/kernelcore/dev/low-level/phoenix-cloud-run`
- **Status**: Migration complete (Jan 2, 2026), needs validation & testing
- **Credits**: R$ 10,079.11 GCP promotional credits to burn

## Tech Stack

- Python 3.13 + Nix Flakes
- GCP: Vertex AI Search, Dialogflow CX, BigQuery
- AI/ML: Mistral-7B, ChromaDB, PyTorch
- CLI: Typer + Rich

## Critical Priorities (ROI-Focused)

### 🔥 Phase 1: VALIDATION (DO FIRST)

1. Test all imports: `python phantom.py --help`
2. Test GCP validation: `python phantom.py gcp validate`
3. Run credit burner smoke test
4. Fix any broken imports IMMEDIATELY

### 🎯 Phase 2: TESTING (HIGH VALUE)

- Create test suite: `phantom/tests/core/`, `phantom/tests/modules/`
- Unit tests for GCP modules (auth, datastores, search)
- Integration test for credit burner workflow
- Target: >80% coverage

### 💰 Phase 3: MONETIZE CREDITS

- Complete knowledge extraction module
- Optimize credit consumption rate
- BigQuery audit validation

## Working Principles

### DO

✅ **Test before building** - No untested code
✅ **Use existing modules** - DRY achieved, keep it that way
✅ **Check docs first** - `ARCHITECTURE.md`, `NEXT_STEPS.md`, `CREDIT_BURNER_DETAILED.md`
✅ **Focus on ROI** - Burn credits efficiently, validate spending
✅ **Type hints always** - Python 3.13 strict typing
✅ **Rich output** - Use `rich` for CLI feedback

### DON'T

❌ **No guessing** - Read code before changing
❌ **No duplicate code** - Extract to `phantom/core/utils/`
❌ **No untested features** - Test-driven development
❌ **No wasted credits** - Validate before load testing
❌ **No vague TODOs** - Specific, actionable items only

## Code Standards

**Imports:**

```python
from phantom.core import gcp  # ✅ Correct
from phantom.core.gcp import auth, datastores, search

# NOT: from gcp import ...  # ❌ Wrong

Error Handling:
from phantom.core.utils.error_handling import safe_execute

result = safe_execute(risky_function, "Context description")

Logging:
from rich.console import Console
console = Console()
console.print("[green]Success![/green]")

File Structure Reference

phantom/
├── core/              # Shared modules (USE THESE)
│   ├── gcp/          # GCP integrations (PRODUCTION READY)
│   ├── extraction/   # Code analysis
│   ├── rag/          # RAG server
│   └── utils/        # Shared utilities
├── modules/          # Application modules
│   ├── credit_burner/  # ✅ Complete
│   ├── knowledge/      # 🚧 TODO
│   └── nixos/          # 🚧 TODO
└── tests/            # ⚠️ EMPTY - CREATE FIRST

Quick Commands

Setup:
nix develop
python phantom.py info

Development:
# Run CLI
python phantom.py gcp validate
python phantom.py credit loadtest --queries 100

# Run tests (once created)
pytest phantom/tests/ -v --cov=phantom

Validation:
# Check imports
python -c "from phantom.core import gcp; print('✅ Imports OK')"

# Type check
mypy phantom/

# Lint
ruff check phantom/

Emergency Rules

If stuck:
1. Read NEXT_STEPS.md first
2. Check existing code in phantom/core/
3. Ask specific questions with file paths

If burning money:
1. STOP immediately
2. Validate the task has clear ROI
3. Estimate credit cost BEFORE running
4. Use BigQuery audit to verify spending

If tests fail:
1. Read the full error message
2. Check phantom/core/utils/error_handling.py
3. Add test case for the bug

Success Metrics

- All imports work (phantom.py --help runs)
- GCP validation passes
- Test coverage >80%
- Credit burner validated with BigQuery
- Zero duplicate code
- All TODOs actionable

Documentation Priority

1. NEXT_STEPS.md - Current roadmap
2. ARCHITECTURE.md - System design
3. CREDIT_BURNER_DETAILED.md - Credit consumption guide
4. Inline docstrings in phantom/core/

---
Remember: This project exists to CONSUME GCP CREDITS efficiently. Every task should either:
1. Validate credit burning works
2. Increase credit consumption rate
3. Ensure audit trail is accurate
4. Extract maximum value from R$ 10k credits

No wasted tokens, no guessing, no untested code.

---

```
