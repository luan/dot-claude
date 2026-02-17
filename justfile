build:
    cd tools/cli && cargo build --release

check:
    cd tools/cli && cargo fmt --check && cargo clippy -- -W clippy::all && cargo test

install:
    cd tools/cli && cargo install --path crates/gitcontext && cargo install --path crates/ck

completions:
    mkdir -p ~/.config/fish/completions
    ck tool completion fish > ~/.config/fish/completions/ck.fish

setup: install completions
    @echo "Installed: claude-gitcontext, ck (fish completions → ~/.config/fish/completions/ck.fish)"
    @echo "Verify: ck tool slug 'hello world'"
