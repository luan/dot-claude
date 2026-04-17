build:
    cd tools && cargo build --release

check:
    cd tools && cargo fmt --check && cargo clippy -- -W clippy::all && cargo test

install:
    cd tools && cargo install --path crates/ct
    claude mcp remove -s user ct 2>/dev/null || true
    claude mcp add -s user ct ct mcp serve

completions:
    mkdir -p ~/.config/fish/completions
    ct tool completion fish > ~/.config/fish/completions/ct.fish

setup: install completions
    @echo "Installed: ct (fish completions → ~/.config/fish/completions/ct.fish)"
    @echo "Verify: ct tool slug 'hello world'"
