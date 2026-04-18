build:
    cargo build --release

check:
    cargo fmt --check && cargo clippy -- -W clippy::all && cargo test

install:
    cargo install --path .
    claude mcp remove -s user ct 2>/dev/null || true
    claude mcp add -s user ct ct mcp serve

completions:
    mkdir -p ~/.config/fish/completions
    ct tool completion fish > ~/.config/fish/completions/ct.fish

setup: install completions
    @echo "Installed: ct (fish completions → ~/.config/fish/completions/ct.fish)"
    @echo "Verify: ct tool slug 'hello world'"
