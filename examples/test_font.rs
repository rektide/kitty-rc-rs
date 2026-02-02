use kitty_rc::{Kitty, commands::SetFontSizeCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut kitty = Kitty::builder()
        .socket_path("/run/user/1000/kitty-757336.sock")
        .password("0pD4cLsKze84eCYOh7dIlvMFF87rgHEPSkngVpgbtYJ9hAzJ")
        .connect()
        .await?;

    println!("Testing set-font-size increment command...");
    
    // Try increment
    let cmd = SetFontSizeCommand::new(0)
        .increment_op("+")
        .build()?;

    let result = kitty.execute(&cmd).await?;
    println!("Increment result: ok={}, data={:?}, error={:?}", result.ok, result.data, result.error);

    Ok(())
}
