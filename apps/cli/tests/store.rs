use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = PathBuf::from(format!(
            "/tmp/opencode/kitty-cli-store-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("kitty")).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("kitty/pubkey.tsv")
    }
    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_kitty-pubkey-db"))
            .env_clear()
            .env("HOME", &self.0)
            .env("XDG_STATE_HOME", &self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn malformed_cleanup_and_get_refuse_without_losing_a_single_byte() {
    for bytes in [
        b"1\t\t1:old\nmalformed\r\n".as_slice(),
        b"1\t\t1:old\t4\nbroken".as_slice(),
        b"bad\xff".as_slice(),
    ] {
        let scratch = Scratch::new();
        fs::write(scratch.path(), bytes).unwrap();
        for args in [&["cleanup", "--quiescent"][..], &["get", "1"][..]] {
            let output = scratch.run(args);
            assert!(!output.status.success());
            assert_eq!(fs::read(scratch.path()).unwrap(), bytes);
        }
    }
}
#[test]
fn append_and_init_preserve_legacy_and_unterminated_bytes() {
    let scratch = Scratch::new();
    let old = b"1\t8\t1:old\r\nmalformed\n2\t\t1:legacy\t10";
    fs::write(scratch.path(), old).unwrap();
    assert!(scratch.run(&["init"]).status.success());
    assert_eq!(fs::read(scratch.path()).unwrap(), old);
    assert!(
        scratch
            .run(&["add", "--pid", "3", "--pubkey", "1:new"])
            .status
            .success()
    );
    let after = fs::read(scratch.path()).unwrap();
    assert!(after.starts_with(old));
    assert!(after[old.len()..].starts_with(b"\n3\t\t1:new\t"));
    assert!(!scratch.run(&["cleanup"]).status.success());
    assert_eq!(fs::read(scratch.path()).unwrap(), after);
}
#[test]
fn cleanup_requires_quiescence_serializes_writers_and_preserves_surviving_line_bytes() {
    let scratch = Scratch::new();
    let kept = format!("{}\t7\t1:alive\r\n", std::process::id());
    fs::write(scratch.path(), format!("{kept}4294967295\t\t1:dead\t1\n")).unwrap();
    fs::write(scratch.path().with_extension("tsv.lock"), "other owner").unwrap();
    let before = fs::read(scratch.path()).unwrap();
    assert!(!scratch.run(&["cleanup", "--quiescent"]).status.success());
    assert!(
        !scratch
            .run(&["add", "--pid", "3", "--pubkey", "1:new"])
            .status
            .success()
    );
    assert_eq!(fs::read(scratch.path()).unwrap(), before);
    fs::remove_file(scratch.path().with_extension("tsv.lock")).unwrap();
    assert!(scratch.run(&["cleanup", "--quiescent"]).status.success());
    assert_eq!(fs::read(scratch.path()).unwrap(), kept.as_bytes());
}

#[test]
fn explicit_store_and_malformed_selected_environment_are_distinct() {
    let scratch = Scratch::new();
    let path = scratch.0.join("declared.tsv");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_kitty-pubkey-db"))
            .env_clear()
            .env("KITTY_PID", "bad-pid")
            .env("KITTY_WINDOW_ID", "bad-window")
            .arg("--store")
            .arg(&path)
            .args(args)
            .output()
            .unwrap()
    };
    assert!(run(&["init"]).status.success());
    let out = run(&["add", "--pubkey", "1:fixture"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("KITTY_PID must be a u32"));
    assert!(fs::read(&path).unwrap().is_empty());
    assert!(
        !run(&["add", "--pid", "7", "--pubkey", "1:fixture"])
            .status
            .success()
    );
    assert!(
        run(&[
            "add",
            "--pid",
            "7",
            "--window-id",
            "8",
            "--pubkey",
            "1:fixture"
        ])
        .status
        .success()
    );
    assert!(run(&["get", "7"]).status.success());
    assert!(!scratch.path().exists());
}
