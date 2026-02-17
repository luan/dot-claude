build:
    cd tools/cli && cargo build --release

check:
    cd tools/cli && cargo fmt --check && cargo clippy -- -W clippy::all && cargo test

install:
    cd tools/cli && cargo install --path crates/gitcontext && cargo install --path crates/wasc

completions:
    mkdir -p ~/.config/fish/completions
    wasc completion fish > ~/.config/fish/completions/wasc.fish

setup: install completions
    @echo "Installed: claude-gitcontext, wasc (fish completions → ~/.config/fish/completions/wasc.fish)"
    @echo "Verify: wasc slug 'hello world'"
