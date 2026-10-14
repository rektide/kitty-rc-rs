use clap::Parser;
use kitty_rc_cli::{control, interface::Cli};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match control::run(Cli::parse()).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kitty-rc: {error}");
            let mut cause = error.source();
            while let Some(source) = cause {
                eprintln!("  caused by: {source}");
                cause = source.source();
            }
            if error.is::<kitty_rc_cli::Interrupted>() {
                std::process::ExitCode::from(130)
            } else {
                std::process::ExitCode::FAILURE
            }
        }
    }
}
