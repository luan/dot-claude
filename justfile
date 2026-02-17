build:
    cd tools/cli && cargo build --release

check:
    cd tools/cli && cargo fmt --check && cargo clippy -- -W clippy::all && cargo test

install:
    cd tools/cli && cargo install --path crates/slug && cargo install --path crates/planfile && cargo install --path crates/phases && cargo install --path crates/gitcontext && cargo install --path crates/wasc

setup: install
    @echo "Installed: claude-slug, claude-planfile, claude-phases, claude-gitcontext, wasc"
    @echo "Verify: claude-slug 'hello world'"
