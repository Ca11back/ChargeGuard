mod commands;
mod output;

use clap::Parser;
use commands::{ChargeGuardClient, Cli, Command};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{}: {error}", error.code());
        std::process::exit(1);
    }
}

async fn run() -> commands::Result<()> {
    let cli = Cli::parse();
    let client = ChargeGuardClient::connect().await?;

    match cli.command {
        Command::List => {
            let devices = client.list_devices().await?;
            output::print_device_list(&devices);
        }
        Command::Current { device } => {
            let device = client.resolve_device(device).await?;
            output::print_current(&device);
        }
        Command::Modes { device } => {
            let device = client.resolve_device(device).await?;
            output::print_modes(&device);
        }
        Command::Set { mode, device } => {
            let device_id = client.resolve_device_id(device).await?;
            let device = client.set_native_mode(&device_id, &mode).await?;
            output::print_set_result(&device);
        }
        Command::Doctor { json } => {
            let diagnostics = client.diagnostics().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&diagnostics)?);
            } else {
                output::print_diagnostics(&diagnostics);
            }
        }
        Command::Raw { device } => {
            let diagnostics = client.diagnostics().await?;
            output::print_raw(&diagnostics, device.as_deref())?;
        }
    }

    Ok(())
}
