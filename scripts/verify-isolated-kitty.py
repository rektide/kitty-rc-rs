#!/usr/bin/env python3
"""Private headless conformance; never selects an ambient human terminal."""
import os, pathlib, subprocess, tempfile, time

root = pathlib.Path.cwd()
with tempfile.TemporaryDirectory(prefix="kitty-rc-live-", dir="/tmp/opencode") as temp:
    temp = pathlib.Path(temp)
    env = dict(os.environ)
    for key in ("KITTY_PUBLIC_KEY", "KITTY_PID", "KITTY_WINDOW_ID", "KITTY_LISTEN_ON", "DISPLAY", "WAYLAND_DISPLAY"):
        env.pop(key, None)
    env.update(XDG_RUNTIME_DIR=str(temp), WAYLAND_DISPLAY="isolated-wayland", LIBGL_ALWAYS_SOFTWARE="1")
    config = temp / "kitty.conf"
    config.write_text("allow_remote_control password\nremote_control_password test-secret\nlinux_display_server wayland\n")
    keyfile = temp / "key"
    socket = temp / "rc.sock"
    log = open(temp / "process.log", "w+")
    weston = subprocess.Popen(["weston", "--backend=headless", "--renderer=pixman", "--fake-seat", "--socket=isolated-wayland", "--no-config"], env=env, stdout=log, stderr=log)
    kitty = None
    try:
        for _ in range(100):
            if (temp / "isolated-wayland").exists(): break
            if weston.poll() is not None: raise RuntimeError("Weston failed")
            time.sleep(.05)
        child = f"import os,pathlib,time;pathlib.Path({str(keyfile)!r}).write_text(os.environ['KITTY_PUBLIC_KEY']);print('isolated conformance');time.sleep(60)"
        kitty = subprocess.Popen(["kitty", "--config", str(config), "--listen-on", "unix:" + str(socket), "python3", "-c", child], env=env, stdout=log, stderr=log)
        for _ in range(100):
            if keyfile.exists() and socket.exists(): break
            if kitty.poll() is not None: raise RuntimeError("kitty failed")
            time.sleep(.05)
        subprocess.run(["cargo", "+1.88.0", "run", "-p", "kitty-rc", "--example", "conformance", "--", str(socket), keyfile.read_text(), "test-secret"], cwd=root, check=True)
    except Exception:
        log.flush(); log.seek(0); print(log.read()); raise
    finally:
        for process in (kitty, weston):
            if process and process.poll() is None:
                process.terminate()
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: process.kill(); process.wait()
        log.close()
