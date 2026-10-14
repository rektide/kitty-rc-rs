//! Test fixture binary standing in for headless Weston: it creates the
//! Wayland socket inside `$XDG_RUNTIME_DIR` and serves until killed.
fn main() {
    let runtime = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR");
    let display = std::env::var("WAYLAND_DISPLAY").expect("WAYLAND_DISPLAY");
    let socket = std::path::Path::new(&runtime).join(&display);
    std::os::unix::net::UnixListener::bind(&socket).expect("bind fake weston socket");
    std::thread::sleep(std::time::Duration::from_secs(120));
}
