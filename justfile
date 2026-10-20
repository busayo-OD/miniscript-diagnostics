_default:
    @just --list

alias b := build
alias f := fmt
alias c := check
alias t := test
alias p := pre-push
alias d := doc

# Build the project
build:
    cargo build

# Format the code
fmt:
    cargo fmt --all

# Check formatting, compilation, and linting
check:
    cargo fmt --all -- --check
    cargo check --all-targets
    cargo clippy --all-targets -- -D warnings

# Run tests
test:
    cargo test

# Check documentation builds without warnings
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Run all checks before pushing
pre-push: check test doc
