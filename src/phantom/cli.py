#!/usr/bin/env python3
import json
import sys
from pathlib import Path

import yaml

# Fast imports for CLI responsiveness
try:
    import typer
    from rich.console import Console
    from rich.panel import Panel
    from rich.table import Table
except ImportError:
    # If basic dependencies are missing, we can't do much
    sys.exit(1)

app = typer.Typer(no_args_is_help=True, add_completion=False)
console = Console()

knowledge_app = typer.Typer(help="Análise e Auditoria")
ops_app = typer.Typer(help="Status Operacional")
rag_app = typer.Typer(help="RAG & Vetores (LangChain)")

app.add_typer(knowledge_app, name="knowledge")
app.add_typer(ops_app, name="ops")
app.add_typer(rag_app, name="rag")


def load_config(config_path: str):
    path = Path(config_path)
    if not path.exists():
        return {}
    with open(path) as f:
        return yaml.safe_load(f) or {}


# ================= GLOBAL COMMANDS =================
@app.command("info")
def info():
    """
    Exibe informações sobre o ambiente Phantom.
    """
    import importlib.util

    console.print(
        Panel("👻 [bold]PHANTOM Framework[/bold] - v0.1.0", border_style="cyan")
    )

    # Check dependencies
    deps = {"GCP Integration": False, "LangChain/RAG": False, "Code Analysis": False}

    if importlib.util.find_spec("google.cloud"):
        deps["GCP Integration"] = True
    if importlib.util.find_spec("langchain"):
        deps["LangChain/RAG"] = True
    if importlib.util.find_spec("tree_sitter"):
        deps["Code Analysis"] = True

    table = Table(show_header=True, header_style="bold magenta")
    table.add_column("Component")
    table.add_column("Status")

    for k, v in deps.items():
        table.add_row(
            k, "[green]Installed[/green]" if v else "[yellow]Missing[/yellow]"
        )

    console.print(table)


@app.command("version")
def version():
    """
    Exibe a versão atual.
    """
    console.print("Phantom CLI v0.1.0")


# ================= KNOWLEDGE =================
@knowledge_app.command("analyze")
def analyze(
    repo_path: str,
    task_context: str = "General Review",
    config_file: str = "./config/repos.yaml",
):
    """
    Extrai AST e gera JSONL.
    Uso: phantom knowledge analyze ./repo "Contexto"
    """
    # Lazy imports
    from phantom.core.extraction.analyze_code import (
        HermeticAnalyzer,
        validate_repository_path,
    )

    target = Path(repo_path).expanduser().resolve()
    validate_repository_path(target)

    # Load config to find hooks for this repo
    config = load_config(config_file)
    repo_config = next(
        (
            r
            for r in config.get("repos", [])
            if Path(r.get("path", "")).resolve() == target
        ),
        {},
    )

    if repo_config:
        console.print(
            f"📖 Configuração encontrada para: [cyan]{repo_config.get('name')}[/cyan]"
        )
        if "context" in repo_config:
            task_context = repo_config["context"]

    console.print(f"🔬 Analisando: [bold]{target.name}[/bold]")
    analyzer = HermeticAnalyzer(config)

    # Pass hooks to analyzer
    result = analyzer.analyze_repo(target, hooks=repo_config.get("hooks"))
    result["metrics"]["task_context"] = task_context

    out = Path("./data/analyzed") / target.name
    out.mkdir(parents=True, exist_ok=True)

    with open(out / "metrics.json", "w") as f:
        json.dump(result["metrics"], f, indent=2)
    with open(out / "artifacts.json", "w") as f:
        json.dump([a.__dict__ for a in result["artifacts"]], f)

    # Append to global JSONL
    jsonl = Path("./data/analyzed/all_artifacts.jsonl")
    mode = "a" if jsonl.exists() else "w"
    with open(jsonl, mode) as f:
        for a in result["artifacts"]:
            doc = {
                "id": f"{target.name}-{a.name}",
                "jsonData": json.dumps(
                    {
                        "title": a.name,
                        "content": a.content,
                        "repo": target.name,
                        "metrics": result["metrics"],
                        "context": task_context,
                    }
                ),
            }
            f.write(json.dumps(doc) + "\n")

    console.print(f"[green]✅ Extraído: {len(result['artifacts'])} artefatos.[/green]")


@knowledge_app.command("batch-analyze")
def batch_analyze(config_file: str = "./config/repos.yaml"):
    """
    Processa todos os repositórios definidos no arquivo de configuração.
    """
    config = load_config(config_file)
    repos = config.get("repos", [])

    if not repos:
        console.print(
            "[yellow]⚠️  Nenhum repositório encontrado na configuração.[/yellow]"
        )
        return

    console.print(f"🚀 Iniciando análise em lote de {len(repos)} repositórios...")

    for repo in repos:
        path = repo.get("path")
        name = repo.get("name")
        priority = repo.get("priority", "medium")

        if not path:
            continue

        console.print(
            f"\n[bold magenta]📦 Processando: {name} ({priority})[/bold magenta]"
        )

        try:
            analyze(path, config_file=config_file)
        except Exception as e:
            console.print(f"[red]❌ Falha ao processar {name}: {e}[/red]")

    console.print("\n[green]✨ Batch Analysis Complete![/green]")


@knowledge_app.command("summarize")
def summarize(repo_name: str):
    path = Path("./data/analyzed") / repo_name
    if not (path / "metrics.json").exists():
        return
    with open(path / "metrics.json") as f:
        m = json.load(f)

    report = [
        f"# REPORT: {repo_name.upper()}",
        f"Context: {m.get('task_context')}",
        f"LoC: {m.get('loc')}",
    ]
    for h in set(m.get("security_hints", []) + m.get("performance_hints", [])):
        report.append(f"- ⚠️ {h}")
    (path / "EXECUTIVE_REPORT.md").write_text("\n".join(report))
    console.print(f"[green]✅ Relatório: {path}/EXECUTIVE_REPORT.md[/green]")


# ================= RAG (LangChain) =================
@rag_app.command("ingest")
def rag_ingest(source_file: str = "./data/analyzed/all_artifacts.jsonl"):
    """
    Cria Vector DB Local (Chroma) para alta precisão.
    """
    try:
        from phantom.core.rag.engine import RigorousRAGEngine
    except ImportError:
        console.print("[red]❌ Dependências LangChain ausentes.[/red]")
        return

    engine = RigorousRAGEngine()
    console.print("🧠 [bold]Ingerindo vetores (VertexAI Embeddings)...[/bold]")
    try:
        count = engine.ingest(source_file)
        console.print(f"[green]✅ Indexado: {count} chunks no ChromaDB Local.[/green]")
    except Exception as e:
        console.print(f"[red]❌ Erro: {e}[/red]")


@rag_app.command("query")
def rag_query(question: str):
    """
    Consulta o RAG Local e exibe métricas de precisão.
    """
    try:
        from phantom.core.rag.engine import RigorousRAGEngine
    except ImportError:
        console.print(
            "[red]❌ Dependências LangChain ausentes. Instale: poetry install[/red]"
        )
        return

    engine = RigorousRAGEngine()
    console.print(f"🔎 Buscando: [italic]{question}[/italic]...")

    result = engine.query_with_metrics(question)

    # Exibir Resposta
    console.print(
        Panel(result["answer"], title="🤖 PHANTOM Answer", border_style="green")
    )

    # Exibir Métricas Rigorosas
    metrics = result["metrics"]
    table = Table(title="RAG Quality Metrics")
    table.add_column("Metric", style="cyan")
    table.add_column("Value", style="magenta")

    table.add_row("Avg Confidence", str(metrics["avg_confidence"]))
    table.add_row("Hit Rate (@k=4)", metrics["hit_rate_k"])
    table.add_row("Top Source", Path(metrics["top_source"]).name)

    console.print(table)


# ================= OPS =================
@ops_app.command("status")
def status():
    data = Path("./data/analyzed")
    if data.exists():
        for d in data.iterdir():
            if d.is_dir():
                console.print(f"Repo: [cyan]{d.name}[/cyan] [✅ Local]")


if __name__ == "__main__":
    app()