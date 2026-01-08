{
  description = "PHANTOM Cerebro - Knowledge Extraction & RAG Framework";

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
          config.allowUnfree = true; # For CUDA if needed
        };

        # Python environment with core dependencies
        pythonEnv = pkgs.python313.withPackages (
          ps: with ps; [
            # Core dependencies
            pydantic
            typer
            rich
            tqdm
            python-dotenv
            pyyaml

            # Development tools
            pytest
            black
            isort
            ruff
          ]
        );

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.poetry
            pkgs.google-cloud-sdk
            pkgs.just
            pkgs.git
            pkgs.stdenv.cc.cc.lib
            pkgs.zlib
            pkgs.llama-cpp # For local model testing
          ]
          ++ (pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.cudaPackages.cudatoolkit
          ]);

          shellHook = ''
            export PYTHONPATH="$PWD/src:$PYTHONPATH"

            # Poetry configuration
            export POETRY_VIRTUALENVS_IN_PROJECT=true
            export POETRY_VIRTUALENVS_CREATE=true

            # System libraries for compiled extensions
            export LD_LIBRARY_PATH="${
              pkgs.lib.makeLibraryPath [
                pkgs.stdenv.cc.cc.lib
                pkgs.zlib
                pkgs.stdenv.cc.libc
              ]
            }:$LD_LIBRARY_PATH"

            # Create necessary directories
            mkdir -p ./data/analyzed ./data/vector_db ./data/reports

            # GCP configuration
            if [ -f ~/.config/gcloud/application_default_credentials.json ]; then
              export GOOGLE_APPLICATION_CREDENTIALS=~/.config/gcloud/application_default_credentials.json
            fi

            # Project environment variables
            export CEREBRO_DATA_DIR="$PWD/data"
            export CEREBRO_VECTOR_DB="$PWD/data/vector_db"
            export GCP_PROJECT_ID="gen-lang-client-0530325234"
            export GCP_REGION="us-central1"

            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "🧠 PHANTOM CEREBRO - Development Environment"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "Python: $(python --version)"
            echo "Poetry: $(poetry --version)"

            # Install poetry dependencies if not already installed
            if [ ! -d ".venv" ] || [ ! -f "pyproject.toml" ]; then
              echo "📥 Setting up Poetry environment..."
              poetry env use python3.12
              poetry install
              source .venv/bin/activate
            elif [ -d ".venv" ]; then
              source .venv/bin/activate
            fi

            echo ""
            echo "Available commands:"
            echo "  cerebro ops health           - Verify GCP, Quotas and Environment"
            echo "  cerebro knowledge analyze .  - Extract AST from current repo"
            echo "  cerebro rag ingest           - Index artifacts to ChromaDB"
            echo "  cerebro rag query \"...\"      - Query the Knowledge Base"
            echo "  just pipeline                - Run full validation (test + lint)"
            echo "  pytest tests/                - Run unit tests"
            echo ""
            echo "Note: Use 'nix develop --command poetry run cerebro ...' for execution."
            echo ""
            echo "Data directories:"
            echo "  ./data/analyzed     - Analyzed artifacts"
            echo "  ./data/vector_db    - Vector database"
            echo "  ./data/reports      - Analysis reports"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
          '';
        };

      }
    );
}
