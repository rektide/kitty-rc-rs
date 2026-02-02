use kitty_rc::protocol::KittyMessage;
use kitty_rc::encryption::Encryptor;
use x25519_dalek::{PublicKey, StaticSecret};

fn main() {
    let secret = StaticSecret::random_from_rng(&mut rand_core::OsRng);
    let public_key = PublicKey::from(&secret);
    
    unsafe {
        std::env::set_var(
            "KITTY_PUBLIC_KEY",
            format!("1:{}", base85::encode(public_key.as_bytes())),
        );
    }
    
    let encryptor = Encryptor::new().unwrap();
    
    let original_message = KittyMessage::new("set-font-size", vec![0, 14, 2])
        .payload(serde_json::json!({"increment_op": "+", "size": 0}));
    
    let message_json = serde_json::to_value(&original_message).unwrap();
    let mut command_json = message_json;
    if let Some(obj) = command_json.as_object_mut() {
        obj.insert("password".to_string(), serde_json::json!("test-password"));
        obj.insert("timestamp".to_string(), serde_json::json!(1234567890000000000i64));
    }
    
    let encrypted = encryptor.encrypt_command(command_json).unwrap();
    println!("Encrypted message structure:");
    let encrypted_str = serde_json::to_string_pretty(&encrypted).unwrap();
    println!("{}", encrypted_str);
    
    let obj = encrypted.as_object().unwrap();
    println!("\nFields present:");
    for key in obj.keys() {
        println!("- {}", key);
    }
    
    let has_expected_fields = vec!["version", "iv", "tag", "pubkey", "encrypted"]
        .iter()
        .all(|k| obj.contains_key(*k));
    println!("\nHas all expected fields (version, iv, tag, pubkey, encrypted): {}", has_expected_fields);
}
