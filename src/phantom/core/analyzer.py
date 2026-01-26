import json
import os
from pathlib import Path
from typing import Dict, List, Optional

import pathspec
from rich.progress import track


class RepoAnalyzer:
    def __init__(self, root_path: str, max_file_size_kb: int = 100):
        self.root_path = Path(root_path).resolve()
        self.max_file_size = max_file_size_kb * 1024
        self.ignore_spec = self._load_gitignore()

        # Padrões padrão para ignorar caso não tenha gitignore ou para reforçar
        self.default_ignores = [
            ".git",
            "__pycache__",
            "node_modules",
            "dist",
            "build",
            "target",
            ".venv",
            ".env",
            ".nix-pip",
            "*.pyc",
            "*.o",
            "*.so",
            "*.lock",
            "package-lock.json",
            "yarn.lock",
            "go.sum",
            "Cargo.lock",
        ]

    def _load_gitignore(self) -> Optional[pathspec.PathSpec]:
        gitignore_path = self.root_path / ".gitignore"
        if gitignore_path.exists():
            with open(gitignore_path, "r") as f:
                return pathspec.PathSpec.from_lines("gitwildmatch", f)
        return None

    def _is_ignored(self, file_path: Path) -> bool:
        # Caminho relativo para check
        rel_path = file_path.relative_to(self.root_path).as_posix()

        # Check 1: Padrões hardcoded
        for pattern in self.default_ignores:
            if pattern in rel_path.split("/"):  # Check diretórios exatos
                return True
            if rel_path.endswith(pattern.replace("*", "")):  # Check extensões simples
                return True

        # Check 2: .gitignore
        if self.ignore_spec and self.ignore_spec.match_file(rel_path):
            return True

        return False

    def _is_binary(self, file_path: Path) -> bool:
        """Heurística simples para detectar binários"""
        try:
            with open(file_path, "rb") as check_file:
                check_file.read(1024)
                return False
        except:
            return True

    def scan(self) -> List[Dict]:
        artifacts = []

        print(f"🔍 Scanning: {self.root_path}")

        # Coleta todos os arquivos primeiro para a barra de progresso
        all_files = []
        for root, dirs, files in os.walk(self.root_path):
            # Filtrar diretórios in-place para não descer neles
            dirs[:] = [d for d in dirs if not self._is_ignored(Path(root) / d)]

            for file in files:
                file_path = Path(root) / file
                if not self._is_ignored(file_path):
                    all_files.append(file_path)

        # Processa arquivos
        for file_path in track(all_files, description="Parsing files..."):
            if file_path.stat().st_size > self.max_file_size:
                continue

            if self._is_binary(file_path):
                continue

            try:
                with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()

                rel_path = file_path.relative_to(self.root_path).as_posix()

                artifacts.append(
                    {
                        "id": rel_path.replace("/", "_").replace(".", "_"),
                        "title": rel_path,
                        "content": content,
                        "metadata": {
                            "path": rel_path,
                            "extension": file_path.suffix,
                            "size": len(content),
                        },
                    }
                )
            except Exception as e:
                print(f"⚠️ Error reading {file_path}: {e}")

        return artifacts

    def save_jsonl(self, artifacts: List[Dict], output_path: str):
        with open(output_path, "w", encoding="utf-8") as f:
            for item in artifacts:
                f.write(json.dumps(item) + "\n")
