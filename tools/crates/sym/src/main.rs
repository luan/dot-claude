use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "sym")]
#[command(about = "Fast code indexer and symbol discovery tool")]
struct Cli {
    #[command(flatten)]
    args: sym::cli::SymArgs,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = sym::run(cli.args) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
