#!/usr/bin/env python3
"""Private headless image/run conformance. Never selects an ambient terminal.

Run under `mise exec -- python3 scripts/verify-isolated-uploads.py` so the example
uses the user's selected Rust. KITTY_RC_TEST_KITTY selects an explicit binary
(otherwise PATH's kitty). Actual version is printed, not mislabeled
as pinned stable/source or an entire supported-version matrix.
"""
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import time
import zlib

def chunk(kind, data):
    return struct.pack(">I",len(data))+kind+data+struct.pack(">I",zlib.crc32(kind+data))

def main():
    kitty_binary=os.environ.get("KITTY_RC_TEST_KITTY","kitty")
    subprocess.run([kitty_binary,"--version"],check=True)
    root=Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="kitty-private-upload-",dir="/tmp/opencode") as folder:
        temp=Path(folder)
        env=dict(os.environ)
        for key in ("KITTY_PUBLIC_KEY","KITTY_PID","KITTY_WINDOW_ID","KITTY_LISTEN_ON","DISPLAY","WAYLAND_DISPLAY"):
            env.pop(key,None)
        env.update(XDG_RUNTIME_DIR=str(temp),WAYLAND_DISPLAY="private-upload-wayland",LIBGL_ALWAYS_SOFTWARE="1")
        config=temp/"kitty.conf"
        config.write_text("allow_remote_control password\nremote_control_password upload-test-secret\nlinux_display_server wayland\n")
        keyfile=temp/"key";socket=temp/"rc.sock";png=temp/"pixel.png"
        # Independent valid 1x1 RGBA PNG with computed CRCs and zlib scanline.
        png.write_bytes(b"\x89PNG\r\n\x1a\n"+chunk(b"IHDR",struct.pack(">IIBBBBB",1,1,8,6,0,0,0))+chunk(b"IDAT",zlib.compress(bytes([0,64,128,255,255])))+chunk(b"IEND",b""))
        with open(temp/"process.log","w+") as log:
            weston=subprocess.Popen(["weston","--backend=headless","--renderer=pixman","--fake-seat","--socket=private-upload-wayland","--no-config"],env=env,stdout=log,stderr=log)
            kitty=None
            try:
                for _ in range(600):
                    if (temp/"private-upload-wayland").exists():break
                    if weston.poll() is not None:raise RuntimeError("private Weston failed")
                    time.sleep(.05)
                child=f"import os,pathlib,time;pathlib.Path({str(keyfile)!r}).write_text(os.environ['KITTY_PUBLIC_KEY']);print('private upload fixture');time.sleep(120)"
                kitty=subprocess.Popen([kitty_binary,"--config",str(config),"--listen-on","unix:"+str(socket),"python3","-c",child],env=env,stdout=log,stderr=log)
                for _ in range(600):
                    if keyfile.exists() and socket.exists():break
                    if kitty.poll() is not None:raise RuntimeError("private kitty failed")
                    time.sleep(.05)
                if not keyfile.exists() or not socket.exists():raise RuntimeError("private kitty startup did not publish key/socket within 30 seconds")
                subprocess.run(["cargo","run","-p","kitty-rc-registry","--example","upload_conformance","--",str(socket),keyfile.read_text(),"upload-test-secret",str(png)],cwd=root,check=True,timeout=90)
            except Exception:
                log.flush();log.seek(0);print(log.read());raise
            finally:
                for process in (kitty,weston):
                    if process and process.poll() is None:
                        process.terminate()
                        try:process.wait(timeout=5)
                        except subprocess.TimeoutExpired:process.kill();process.wait()

if __name__=="__main__":main()
