use clap::{Parser, Subcommand};
mod compile;
mod refactor;
mod utils;
#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Refactor command, it is capable of reordering and deduplicating the `thebibliography` environment, according to the content of the .tex document
    Refactor(refactor::RefactorCli),
    /// Compile command, it is capable of turning a BibTeX file into a `thebibliography` environment.
    Compile(compile::CompileCli),
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Compile(cli) => {
            compile::run_compile(cli);
        }
        Commands::Refactor(cli) => {
            refactor::run_refactor(cli);
        }
    }
}
