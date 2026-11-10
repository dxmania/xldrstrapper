use std::{fs, path::PathBuf, process::ExitCode, time::Duration};

use clap::{Parser, Subcommand};
use xldrstrapper::{backend_for, usb, BootPlan, Error, KNOWN_IDS};

#[derive(Parser)]
#[command(about = "Boot OMAP devices over USB")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Boot {
        first_stage: PathBuf,
        image: Option<PathBuf>,
        #[arg(long, default_value_t = 60)]
        wait_secs: u64,
    },
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.cmd {
        Cmd::Boot { first_stage, image, wait_secs } => {
            let plan = BootPlan {
                first_stage: fs::read(&first_stage)?,
                image: image.map(fs::read).transpose()?,
            };

            eprintln!("waiting for a device in USB boot mode...");
            let (id, mut transport) =
                usb::wait_for_device(KNOWN_IDS, Duration::from_secs(wait_secs))?;

            let backend = backend_for(id).ok_or(Error::UnsupportedDevice(id))?;
            eprintln!("found {id}, using backend: {}", backend.name());

            backend.boot(&mut transport, &plan)?;
            eprintln!("done");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}