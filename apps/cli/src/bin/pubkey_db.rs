use clap::Parser;
use kitty_rc_cli::store;

fn main() -> std::process::ExitCode {
    match store::run(store::Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kitty-pubkey-db: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
