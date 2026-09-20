# merle — common tasks. Run `just` to list them.

default:
    @just --list

# Build the whole workspace
build:
    cargo build --workspace --all-targets

# Format all crates
fmt:
    cargo fmt --all

# Check formatting without writing (CI gate)
fmt-check:
    cargo fmt --all --check

# Clippy over every target, warnings are errors (CI gate)
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the test suite
test:
    cargo nextest run --workspace --no-tests=pass

# Licenses and security advisories
deny:
    cargo deny check

# Spell check
typos:
    typos

# Everything CI runs
ci: fmt-check lint test deny typos

# Watch the workspace with bacon
watch:
    bacon clippy-all

# Run the culler
app *ARGS:
    cargo run -p merle-app -- {{ARGS}}

# Run the minimal viewer
view *ARGS:
    cargo run -p merle-view -- {{ARGS}}
