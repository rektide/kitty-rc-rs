//! Test fixture binary standing in for the `kitty-rc` CLI binary in smoke
//! journeys: it accepts the real CLI's global flags and answers the four
//! journey subcommands with consistent private-fixture data.
//! `FAKE_CLI_MODE=fail` makes every query fail.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if std::env::var("FAKE_CLI_MODE").as_deref() == Ok("fail") {
        // Echo the test password so callers can prove redaction.
        if let Ok(password) = std::env::var("MATRIX_TEST_PASSWORD") {
            eprintln!("fake cli: configured failure with {password}");
        } else {
            eprintln!("fake cli: configured failure");
        }
        std::process::exit(4);
    }
    // Positionals are arguments that are neither flags nor flag values.
    let mut positional: Vec<&String> = Vec::new();
    let mut previous_flag: Option<&String> = None;
    for argument in &args {
        if argument.starts_with("--") {
            previous_flag = Some(argument);
        } else if previous_flag.is_some_and(|flag| flag != "--json") {
            previous_flag = None; // this argument was the flag's value
        } else {
            positional.push(argument);
        }
    }
    let subcommand = positional.first().map(|value| value.as_str()).unwrap_or("");
    let value_of = |flag: &str| -> Option<String> {
        args.iter()
            .position(|argument| argument == flag)
            .and_then(|index| args.get(index + 1))
            .cloned()
    };
    // Global flags the smoke helper passes must all be recognized.
    for expected in [
        "--to",
        "--auth",
        "--max-response",
        "--key-file",
        "--password-env",
    ] {
        if value_of(expected).is_none() && args.iter().any(|argument| argument == expected) {
            eprintln!("fake cli: missing value for {expected}");
            std::process::exit(2);
        }
    }
    match subcommand {
        "ls" => {
            println!(
                "{}",
                serde_json::json!([{"id": 7, "tab_id": 3, "title": "fake fixture window"}])
            );
        }
        "active" => {
            println!("{}", serde_json::json!({"id": 7, "tab_id": 3}));
        }
        "goto" | "watch" => {
            if subcommand == "goto" {
                println!("focused 7");
            } else {
                println!("{}", serde_json::json!({"active": 7}));
                println!("{}", serde_json::json!({"active": 7}));
            }
        }
        other => {
            eprintln!("fake cli: unexpected subcommand {other:?}");
            std::process::exit(2);
        }
    }
}
