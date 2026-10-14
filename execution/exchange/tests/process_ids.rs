use std::process::Command;

#[test]
fn emit_process_ids() {
    if std::env::var_os("KITTY_RC_ID_PROBE").is_some() {
        for _ in 0..3 {
            println!("REMOTEID {}", kitty_rc_exchange::remote_request_id());
        }
    }
}

#[test]
fn fresh_process_clients_do_not_restart_remote_ids() {
    let run = || {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "emit_process_ids", "--nocapture"])
            .env("KITTY_RC_ID_PROBE", "1")
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .filter_map(|line| line.strip_prefix("REMOTEID ").map(str::to_owned))
            .collect::<Vec<_>>()
    };
    let a = run();
    let b = run();
    assert_eq!(a.len(), 3);
    assert_eq!(b.len(), 3);
    assert!(a.iter().all(|id| id.len() == 64 && !b.contains(id)));
}
