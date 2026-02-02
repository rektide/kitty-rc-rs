use clap::{Parser, Subcommand};
use kitty_rc::{Kitty, KittyError};

#[derive(Parser)]
#[command(name = "kitty-rc")]
#[command(about = "Remote control utility for kitty terminal emulator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all windows in kitty
    #[command(aliases = ["ls", "list"])]
    ListWindows,
    /// Display the currently active window
    #[command(aliases = ["active", "a"])]
    ActiveWindow,
    /// Switch focus to a specific window
    #[command(aliases = ["g"])]
    Goto {
        /// Window ID to switch to
        window_id: u64,
    },
    /// Watch and print active window changes
    #[command(aliases = ["w"])]
    Watch,
}

#[tokio::main]
async fn main() -> Result<(), KittyError> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListWindows => {
            handle_list_windows().await?;
        }
        Commands::ActiveWindow => {
            handle_active_window().await?;
        }
        Commands::Goto { window_id } => {
            handle_goto(*window_id).await?;
        }
        Commands::Watch => {
            handle_watch().await?;
        }
    }

    Ok(())
}

async fn handle_list_windows() -> Result<(), KittyError> {
    println!("Connecting to kitty at ./kitty.socket...");

    let mut kitty = Kitty::builder()
        .socket_path("./kitty.socket")
        .connect()
        .await?;

    println!("Connected! Listing windows...\n");

    let cmd = kitty_rc::LsCommand::new().build()?;
    let response = kitty.execute(&cmd).await?;

    println!("Response ok: {}", response.ok);

    if let Some(data) = response.data {
        let parsed_data = if let Some(s) = data.as_str() {
            serde_json::from_str(s).unwrap_or(data.clone())
        } else {
            data
        };

        if let Some(os_instances) = parsed_data.as_array() {
            println!("\n=== OS Instances: {} ===\n", os_instances.len());

            for instance in os_instances {
                if let Some(obj) = instance.as_object() {
                    if let Some(tabs) = obj.get("tabs").and_then(|v| v.as_array()) {
                        println!("Tab count: {}", tabs.len());

                        for tab in tabs {
                            if let Some(tab_obj) = tab.as_object() {
                                if let Some(windows) =
                                    tab_obj.get("windows").and_then(|v| v.as_array())
                                {
                                    for window in windows {
                                        if let Some(win_obj) = window.as_object() {
                                            println!("--- Window ---");

                                            if let Some(id) =
                                                win_obj.get("id").and_then(|v| v.as_u64())
                                            {
                                                println!("  Window ID: {}", id);
                                            }

                                            if let Some(title) =
                                                win_obj.get("title").and_then(|v| v.as_str())
                                            {
                                                println!("  Title: {}", title);
                                            }

                                            if let Some(pid) =
                                                win_obj.get("pid").and_then(|v| v.as_u64())
                                            {
                                                println!("  Shell PID: {}", pid);
                                            }

                                            if let Some(cwd) =
                                                win_obj.get("cwd").and_then(|v| v.as_str())
                                            {
                                                println!("  CWD: {}", cwd);
                                            }

                                            if let Some(cmdline) =
                                                win_obj.get("cmdline").and_then(|v| v.as_array())
                                            {
                                                if let Some(cmd) =
                                                    cmdline.get(0).and_then(|v| v.as_str())
                                                {
                                                    println!("  Shell: {}", cmd);
                                                }
                                            }

                                            if let Some(procs) = win_obj
                                                .get("foreground_processes")
                                                .and_then(|v| v.as_array())
                                            {
                                                for proc in procs {
                                                    if let Some(proc_obj) = proc.as_object() {
                                                        println!("  Foreground Process:");

                                                        if let Some(pid) = proc_obj
                                                            .get("pid")
                                                            .and_then(|v| v.as_u64())
                                                        {
                                                            println!("    PID: {}", pid);
                                                        }

                                                        if let Some(proc_cmdline) = proc_obj
                                                            .get("cmdline")
                                                            .and_then(|v| v.as_array())
                                                        {
                                                            if let Some(first_arg) = proc_cmdline
                                                                .get(0)
                                                                .and_then(|v| v.as_str())
                                                            {
                                                                println!("    Name: {}", first_arg);
                                                            }
                                                        }

                                                        if let Some(proc_cwd) = proc_obj
                                                            .get("cwd")
                                                            .and_then(|v| v.as_str())
                                                        {
                                                            println!("    CWD: {}", proc_cwd);
                                                        }
                                                    }
                                                    println!();
                                                }
                                            }

                                            println!();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(error) = response.error {
        println!("\nError: {}", error);
    }

    kitty.close().await?;
    Ok(())
}

async fn handle_active_window() -> Result<(), KittyError> {
    println!("Active window command not yet implemented");
    Ok(())
}

async fn handle_goto(window_id: u64) -> Result<(), KittyError> {
    println!("Goto window {} command not yet implemented", window_id);
    Ok(())
}

async fn handle_watch() -> Result<(), KittyError> {
    println!("Watch command not yet implemented");
    Ok(())
}
