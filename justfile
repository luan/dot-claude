build:
    cd tools && cargo build --release

check:
    cd tools && cargo fmt --check && cargo clippy -- -W clippy::all && cargo test

install:
    cd tools && cargo install --path crates/ck

completions:
    mkdir -p ~/.config/fish/completions
    ck tool completion fish > ~/.config/fish/completions/ck.fish

setup: install completions
    @echo "Installed: ck (fish completions → ~/.config/fish/completions/ck.fish)"
    @echo "Verify: ck tool slug 'hello world'"
