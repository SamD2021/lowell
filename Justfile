# Justfile for lowell
# Docs: https://just.systems/man/en/  (recipes, settings, dotenv, etc.)

# --- Settings ---------------------------------------------------------------

# Use bash with strict flags so failures fail fast.
# (You can override with `just --shell zsh --shell-arg -c …` if needed.)
# Ref: Configuring the shell. 
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Load .env automatically so UKI_PATH / RUST_LOG can be set there.
# Ref: dotenv settings.
set dotenv-load

# Default recipe: list everything when you run plain `just`.
# Ref: default recipe pattern.
default:
  @just --list

# --- Variables --------------------------------------------------------------

# Package names
CLI_PKG    := "lowell-cli"

# Binary path (release)
BIN        := "target/release/lowell"

# Log level for CLI unless RUST_LOG is set outside
RUST_LOG   := env("RUST_LOG", "info")

# Optional extra cargo flags (e.g., `-Z sparse-registry`, `--locked`)
CARGO_FLAGS := ""

# --- Core tasks -------------------------------------------------------------

# Format all crates
fmt:
  cargo fmt --all --check

# Lint all targets with warnings as errors
clippy:
  cargo clippy --all-targets --all-features -- -D warnings

# Run tests (unit & doc tests)
test:
  cargo test --all --locked

# Build release CLI
build:
  cargo build -q -p {{CLI_PKG}} --release --locked {{CARGO_FLAGS}}

lowell *ARGS:
  cargo run -p {{CLI_PKG}} --release -- {{ARGS}}

# --- Inspect helpers --------------------------------------------------------

# Run: just inspect ../uki-out/vmlinuz-virt.efi
uki-inspect uki *ARGS: build
  {{BIN}} --log-level {{RUST_LOG}} uki inspect {{uki}} {{ARGS}}

# Use UKI_PATH from env/.env:
#   echo 'UKI_PATH=../uki-out/vmlinuz-virt.efi' > .env
#   just inspect-env
uki-inspect-env: build
  {{BIN}} --log-level {{RUST_LOG}} uki inspect "${UKI_PATH:?Set UKI_PATH in env or .env}"

# --- CI aggregate -----------------------------------------------------------

# What the GitHub Action should run
ci: fmt clippy test

# --- Smoke test (ignored) ---------------------------------------------------

# Runs the ignored real-UKI smoke test
# Usage:
#   just smoke                  # uses $UKI_PATH from env/.env
#   just smoke ../uki-out/uki.efi  # overrides $UKI_PATH
smoke uki='':
  IN="{{uki}}"; [[ -n "$IN" ]] || IN="${UKI_PATH:-}"; \
  : "${IN:?error: provide a UKI path (just smoke /path/to/uki) or set UKI_PATH in env/.env}"; \
  [[ "$IN" = /* ]] || IN="$(cd -- "$(dirname -- "$IN")" && pwd -P)/$(basename -- "$IN")"; \
  echo "Using UKI_PATH=$IN"; \
  UKI_PATH="$IN" cargo test -p lowell-core -- --ignored

# --- Speed checks (optional) ------------------------------------------------

# quick, release-mode JSON inspect (good for timing locally)
#   just fast ../uki-out/vmlinuz-virt.efi
fast uki: build
  {{BIN}} --log-level {{RUST_LOG}} uki inspect {{uki}} --format json

