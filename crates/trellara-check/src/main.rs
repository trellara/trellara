use clap::Parser;
use trellara_check::{run_check, CheckArgs, CheckError, Result};

#[tokio::main]
async fn main() {
    if let Err(error) = command_main().await {
        eprintln!("trellara-check: {error}");
        std::process::exit(2);
    }
}

async fn command_main() -> Result<()> {
    let args = CheckArgs::parse();
    let rendered = run_check(&args).await?;
    if let Some(output) = &args.output {
        std::fs::write(output, rendered).map_err(|source| CheckError::WriteOutput {
            path: output.display().to_string(),
            source,
        })?;
        println!(
            "wrote trellara-check {} report to {}",
            args.format,
            output.display()
        );
    } else {
        println!("{rendered}");
    }
    Ok(())
}
