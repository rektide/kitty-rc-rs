//! Schema/frame/protection leaf entrances: no runtime, files, sockets or facade.
use clap::Parser;
use kitty_rc_crypto::{ServerPublicKey, encrypt_raw_request};
use kitty_rc_frame::{FrameDecoder, frame};
use kitty_rc_schema::PlainRequest;
use rand_core::OsRng;

#[derive(Parser)]
#[command(about = "Encode and incrementally decode an offline ls request")]
struct Args {
    /// Optional explicit published key (1:base85); no server is contacted.
    #[arg(long, requires_all = ["password", "timestamp_ns"])]
    key: Option<String>,
    /// Offline specimen only: shell arguments are not safe secret storage.
    #[arg(long, requires = "key")]
    password: Option<String>,
    /// Caller-provided epoch nanoseconds; this example never reads a clock.
    #[arg(long, requires = "key")]
    timestamp_ns: Option<u64>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let request = PlainRequest::new("ls");
    let payload = match (args.key, args.password, args.timestamp_ns) {
        (Some(key), Some(password), Some(timestamp)) => serde_json::to_vec(&encrypt_raw_request(
            serde_json::to_value(request)?,
            &password,
            timestamp,
            &ServerPublicKey::parse(&key)?,
            &mut OsRng,
        )?)?,
        (None, None, None) => serde_json::to_vec(&request)?,
        _ => return Err("encrypted specimen requires key, password and timestamp".into()),
    };
    let bytes = frame(&payload);
    let mut decoder = FrameDecoder::new(16 * 1024);
    for fragment in bytes.chunks(7) {
        decoder.extend(fragment)?;
    }
    let recovered = decoder.next_frame()?.ok_or("missing complete frame")?;
    decoder.finish()?;
    println!("{}", String::from_utf8(recovered)?);
    Ok(())
}
