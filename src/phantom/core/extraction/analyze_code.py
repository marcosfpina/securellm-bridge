# src/phantom/core/extraction/analyze_code.py
"""
PHANTOM Deep Intelligence Engine
- Advanced AST Analysis
- Dependency Mapping
- Security & Performance Heuristics
"""

import ast
import fnmatch
import json
import re
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Dict, List, Optional

from tree_sitter_languages import get_language, get_parser


@dataclass
class RepoMetrics:
    name: str
    loc: int = 0
    functions: int = 0
    classes: int = 0
    dependencies: List[str] = None
    complexity_score: int = 0
    security_hints: List[str] = None
    performance_hints: List[str] = None
    task_context: Optional[str] = None
    hook_results: Dict[str, str] = None


def validate_repository_path(repo_path: Path):
    """Garante que o caminho existe e é um diretório."""
    if not repo_path.exists():
        raise ValueError(f"Repository path does not exist: {repo_path}")
    if not repo_path.is_dir():
        raise ValueError(f"Repository path is not a directory: {repo_path}")


@dataclass
class CodeArtifact:
    repo: str
    file_path: str
    artifact_type: str
    name: str
    content: str
    context: str
    dependencies: List[str]
    documentation: str
    metadata: Dict


class HermeticAnalyzer:
    def __init__(self, config: Dict = None):
        self.config = config or {}
        self.parsers = {}
        for lang in ["python", "nix", "rust", "bash", "typescript", "javascript"]:
            try:
                self.parsers[lang] = get_parser(lang)
            except Exception:
                pass

    def run_hooks(
        self, stage: str, hooks_config: List[Dict], repo_path: Path
    ) -> Dict[str, str]:
        results = {}
        if not hooks_config:
            return results

        print(f"🪝  Running {stage} hooks...")
        for hook in hooks_config:
            cmd = hook.get("command")
            desc = hook.get("description", cmd)
            allow_fail = hook.get("allow_failure", False)

            if not cmd:
                continue

            print(f"   ▶ {desc}")
            try:
                # Run in the repo directory
                res = subprocess.run(
                    cmd, shell=True, cwd=str(repo_path), capture_output=True, text=True
                )
                status = "✅" if res.returncode == 0 else "❌"
                results[desc] = f"{status} (Exit: {res.returncode})"

                if res.returncode != 0 and not allow_fail:
                    print(f"   ❌ Failed: {res.stderr.strip()}")
                elif res.returncode == 0:
                    print(f"   ✅ Success")

            except Exception as e:
                results[desc] = f"❌ Error: {str(e)}"
                print(f"   ❌ Exception: {e}")

        return results

    def detect_language(self, file_path: Path) -> Optional[str]:
        mapping = {
            ".py": "python",
            ".nix": "nix",
            ".rs": "rust",
            ".sh": "bash",
            ".ts": "typescript",
            ".js": "javascript",
        }
        return mapping.get(file_path.suffix.lower())

    def analyze_repo(self, repo_path: Path, hooks: Optional[Dict] = None) -> Dict:
        """Realiza análise profunda de um repositório completo"""
        artifacts = []
        metrics = RepoMetrics(
            name=repo_path.name,
            dependencies=[],
            security_hints=[],
            performance_hints=[],
            hook_results={},
        )

        # 0. Run Pre-Analyze Hooks
        if hooks and "pre_analyze" in hooks:
            pre_results = self.run_hooks("pre_analyze", hooks["pre_analyze"], repo_path)
            metrics.hook_results.update(pre_results)

        # 1. Analisar Dependências (Filesystem Scan)
        metrics.dependencies = self.extract_external_deps(repo_path)

        # Global excludes
        excludes = self.config.get("global", {}).get("exclude", [])
        # Always exclude .git and common ignores
        excludes.extend(
            [
                ".git",
                ".venv",
                "__pycache__",
                "node_modules",
                ".pytest_cache",
                ".nix-pip",
            ]
        )

        # 2. Analisar Código
        for file_path in repo_path.rglob("*"):
            # Check exclusions
            if any(
                fnmatch.fnmatch(part, pat)
                for part in file_path.parts
                for pat in excludes
            ):
                continue

            if file_path.is_file() and not any(
                part.startswith(".") for part in file_path.parts
            ):
                lang = self.detect_language(file_path)
                content = file_path.read_text(errors="ignore")
                metrics.loc += len(content.splitlines())

                if lang in self.parsers:
                    file_artifacts = self.analyze_file(file_path, lang, content)
                    artifacts.extend(file_artifacts)

                    # Atualizar métricas de contagem
                    metrics.functions += sum(
                        1 for a in file_artifacts if a.artifact_type == "function"
                    )
                    metrics.classes += sum(
                        1 for a in file_artifacts if a.artifact_type == "class"
                    )

                    # Heurísticas de Segurança e Performance
                    self.check_heuristics(content, metrics)

        # 3. Run Post-Analyze Hooks
        if hooks and "post_analyze" in hooks:
            post_results = self.run_hooks(
                "post_analyze", hooks["post_analyze"], repo_path
            )
            metrics.hook_results.update(post_results)

        return {"artifacts": artifacts, "metrics": asdict(metrics)}

    def extract_external_deps(self, repo_path: Path) -> List[str]:
        deps = []
        dep_files = {
            "requirements.txt": r"^([a-zA-Z0-9\-_]+)",
            "pyproject.toml": r'([a-zA-Z0-9\-_]+)\s*=\s*["\'{]',
            "package.json": r'"([a-zA-Z0-9\-_@\/]+)"\s*:',
            "Cargo.toml": r"^([a-zA-Z0-9\-_]+)\s*=",
        }
        for file_name, pattern in dep_files.items():
            f_path = repo_path / file_name
            if f_path.exists():
                content = f_path.read_text(errors="ignore")
                found = re.findall(pattern, content, re.MULTILINE)
                deps.extend(found)
        return list(set(deps))

    def check_heuristics(self, content: str, metrics: RepoMetrics):
        # Segurança
        sec_patterns = {
            "hardcoded_key": r'(?i)(key|secret|password|token)\s*=\s*["\"][a-zA-Z0-9]{10,}',
            "unsafe_exec": r"(os\.system|subprocess\.run|eval|exec)\(",
        }
        for key, p in sec_patterns.items():
            if re.search(p, content):
                metrics.security_hints.append(key)

        # Performance
        perf_patterns = {
            "no_cache": r"(?i)(cache=False|no-cache)",
            "heavy_loop": r"for .* in range\(len\(.*\)\)",
        }
        for key, p in perf_patterns.items():
            if re.search(p, content):
                metrics.performance_hints.append(key)

    def analyze_file(
        self, file_path: Path, lang: str, content: str
    ) -> List[CodeArtifact]:
        artifacts = []
        if lang == "python":
            try:
                ast_tree = ast.parse(content)
                for node in ast.walk(ast_tree):
                    if isinstance(
                        node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)
                    ):
                        a_type = (
                            "function"
                            if not isinstance(node, ast.ClassDef)
                            else "class"
                        )
                        artifacts.append(
                            CodeArtifact(
                                repo=repo_name(file_path),
                                file_path=str(file_path),
                                artifact_type=a_type,
                                name=node.name,
                                content=ast.get_source_segment(content, node) or "",
                                context="",
                                dependencies=[],
                                documentation=ast.get_docstring(node) or "",
                                metadata={"line": node.lineno},
                            )
                        )
            except:
                pass
        return artifacts


def repo_name(file_path: Path) -> str:
    for p in ["projects", "Projects", "low-level"]:
        if p in file_path.parts:
            idx = file_path.parts.index(p)
            return (
                file_path.parts[idx + 1] if len(file_path.parts) > idx + 1 else "root"
            )
    return file_path.parts[0] if file_path.parts else "unknown"
