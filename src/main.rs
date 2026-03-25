use clap::Parser;
use veln::cli::Cli;

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    veln::application::run(cli.command).map_err(miette::Report::new)
}
