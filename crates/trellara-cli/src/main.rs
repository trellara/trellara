use clap::Parser;
use trellara_cli::{execute, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match execute(cli).await {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
