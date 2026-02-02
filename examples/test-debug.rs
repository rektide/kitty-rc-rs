use kitty_rc::{Kitty, command::CommandBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test without password
    println!("=== Testing without password ===");
    let mut kitty = Kitty::builder()
        .socket_path("/run/user/1000/kitty-101555.sock")
        .connect()
        .await?;

    let cmd = CommandBuilder::new("ls").build();
    println!("Command: {:?}", cmd);
    let result = kitty.execute(&cmd).await;
    println!("Result: {:?}", result);

    Ok(())
}
