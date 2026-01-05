{
  description = "PHANTOM - Unified Framework with Poetry & Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        python = pkgs.python312;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            python
            pkgs.poetry
            pkgs.google-cloud-sdk
            pkgs.just
            pkgs.git
            pkgs.stdenv.cc.cc.lib
            pkgs.zlib
          ];

          shellHook = ''
            export PYTHONPATH="$PWD/src:$PYTHONPATH"
            
            # Garante que o Poetry crie o venv dentro do projeto para fácil inspeção
            export POETRY_VIRTUALENVS_IN_PROJECT=true
            export POETRY_VIRTUALENVS_CREATE=true

            # Garante bibliotecas do sistema para extensões compiladas (torch, tree-sitter)
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib pkgs.zlib ]}:$LD_LIBRARY_PATH"

            echo "📥 Sincronizando dependências com Poetry (Python 3.12)..."
            
            # Força o uso do Python 3.12
            poetry env use python3.12
            
            if ! poetry install; then
              echo "⚠️  Falha no poetry install. Tentando com --no-root se necessário..."
              poetry install --no-root
            fi

            # Ativa o venv do poetry automaticamente no shell do nix
            if [ -d ".venv" ]; then
              source .venv/bin/activate
            fi

            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "🧠 PHANTOM - Poetry + Nix Environment (Py3.12)"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "Entrypoint: phantom [comando]"
            echo ""
            echo "Comandos disponíveis:"
            echo "  phantom knowledge analyze --path ./src"
            echo "  phantom knowledge summarize"
            echo "  phantom gcp validate"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
          '';
        };
      }
    );
}