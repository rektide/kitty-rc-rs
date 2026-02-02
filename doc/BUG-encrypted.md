Problem
The kitty-rc library's encrypt_command() function incorrectly inserts password and timestamp fields inside the payload object instead of at the top level of the JSON request object.
According to the kitty remote control protocol (https://sw.kovidgoyal.net/kitty/rc_protocol/#encrypted-communication), when sending encrypted commands, the structure should be:
{
version: [0, 14, 2],
iv: ...,
tag: ...,
pubkey: ...,
encrypted: The original command encrypted and base85 encoded
}
The password and timestamp fields should be at the top level (siblings to cmd, version, etc.), NOT inside the payload object.
However, kitty-rc's encrypt_command() implementation does this:
fn encrypt_command(&self, mut message: KittyMessage) -> Result<KittyMessage, KittyError> {
// ... gets password and timestamp ...

    if let Some(payload) = &mut message.payload {
        if let Some(obj) = payload.as_object_mut() {
            // BUG: inserts password/timestamp INSIDE payload object
            obj.insert("password", serde_json::json!(password));
            obj.insert("timestamp", serde_json::json!(timestamp));
        }
    }

    // ... encrypts entire payload and replaces it
    let encrypted_payload = encryptor.encrypt_command(message.payload.unwrap())?;
    message.payload = Some(encrypted_payload);

}
This breaks kitty's protocol parser.
Evidence
From kitty-rc v0.4.2 source:

- File: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/kitty-rc-0.4.1/src/client.rs
- The encrypt_command() function adds password and timestamp to the payload object
  From zooming-kittens logs:
- Commands sent by kitty-focus-tracker show encrypted payload structure
- Example: {"cmd": "set-font-size", "version": [0, 14, 2], "payload": {"increment_op": "+", "size": 0}, "iv": "...", "tag": "...", "pubkey": "...", "encrypted": "..."}
  The iv, tag, pubkey, and encrypted fields should be at the top level of the request, wrapping the payload object.
  Impact
  The zooming-kittens application is sending commands that kitty cannot parse correctly because:

1. The encrypted payload is wrapped in another layer: {"encrypted": "...", "payload": {cmd, version, payload}}
2. kitty expects the password and timestamp at the top level: {cmd, version, password, timestamp, encrypted: {cmd, version, payload}}
   This causes all set-font-size commands to fail with: "Remote control is disabled. Add allow_remote_control to your kitty.conf"
   Workaround
   The zooming-kittens code currently uses encrypted commands. To work around this bug:
3. The encrypt_command() function should be fixed to add password and timestamp at the top level of the JSON request
4. Alternatively, the code could be modified to:
   - Remove the encryption for simple commands like set-font-size
   - Or add a flag to disable encryption for certain commands
     Recommended Action
     This bug should be reported to the kitty-rc repository with:
5. Description of the problem
6. Evidence from protocol docs and test output
7. Suggested fix
8. Workaround options for affected users
   Repository: https://github.com/kovidgoyal/kitty-rc
