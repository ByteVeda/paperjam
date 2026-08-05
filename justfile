# paperjam — common developer commands
#
# Run `just` (no args) for the list. Requires `just` (https://just.systems).

default:
    @just --list

# --- Build ---------------------------------------------------------------

# Build + install the Python extension into the current venv (release).
build:
    uv run maturin develop --release

# Build + install the Python extension in debug mode (fast rebuild).
build-dev:
    uv run maturin develop

# Build the whole Rust workspace.
build-rust:
    cargo build --workspace

# Compile-check the wasm target (no linking).
build-wasm:
    cargo check -p paperjam-wasm --target wasm32-unknown-unknown

# Build the Docusaurus docs site.
build-docs:
    cd docs-site && npm ci && npm run build

# Render crate API docs into target/doc.
rustdoc:
    cargo doc --workspace --no-deps

# --- Test ----------------------------------------------------------------

# Run the full Rust test suite.
test-rust:
    cargo test --workspace

# Run the Python test suite (requires `just build` first).
test-py:
    uv run pytest tests/python/ -v

# Run both Rust and Python test suites.
test: test-rust test-py

# Run the table-accuracy harness (slower, requires fixtures).
test-accuracy:
    uv run pytest tests/python/ -m accuracy -v

# --- Lint / format -------------------------------------------------------

# Apply all autoformatters and run every pre-commit hook.
check:
    pre-commit run --all-files

# Rust-only checks (fmt + clippy).
check-rust:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

# Python-only checks (ruff + mypy).
check-py:
    uv run ruff check py_src/ tests/
    uv run ruff format --check py_src/ tests/
    uv run mypy py_src/ tests/ examples/ --ignore-missing-imports

# Apply autoformatters (doesn't fail on remaining lint issues).
fmt:
    cargo fmt --all
    uv run ruff format py_src/ tests/
    uv run ruff check --fix py_src/ tests/

# --- Clean ---------------------------------------------------------------

# Remove Rust build artifacts.
clean:
    cargo clean

# Remove Rust build artifacts and Python caches.
clean-all: clean
    rm -rf .pytest_cache .mypy_cache .ruff_cache
    find . -type d -name __pycache__ -exec rm -rf {} +
