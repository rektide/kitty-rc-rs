#!/usr/bin/env python3
"""Execute pinned upload constructors without importing native kitty.

Reads GPL source as the independent oracle, does not copy/translate its algorithm
into library code. Tests observable wire contracts through actual AST methods.
No terminal/server I/O; temporary local fixture only.
"""
import ast
import base64
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace

REV = "b2cb871254fbdac19a085188813c8dfb258b95cd"

def extract(checkout, path):
    text = subprocess.check_output(["git", "-C", checkout, "show", f"{REV}:{path}"], text=True)
    method = next(n for n in ast.walk(ast.parse(text)) if isinstance(n, ast.FunctionDef) and n.name == "message_to_kitty")
    module = ast.Module(body=[ast.ImportFrom(module="__future__", names=[ast.alias(name="annotations")], level=0), method], type_ignores=[])
    ast.fix_missing_locations(module)
    scope = {"os": os, "sys": sys, "standard_b64encode": base64.standard_b64encode, "is_png": lambda p: Path(p).read_bytes().startswith(b"\x89PNG\r\n\x1a\n")}
    exec(compile(module, "<actual pinned constructor>", "exec"), scope)
    return scope["message_to_kitty"]

def main(checkout):
    body = b"\x89PNG\r\n\x1a\n" + bytes(range(256)) * 17
    with tempfile.TemporaryDirectory(prefix="kitty-upload-source-", dir="/tmp/opencode") as temp:
        path = Path(temp) / "fixture.png"
        path.write_bytes(body)
        for name, opts, metadata in [
            ("set_background_image", {"match": "title:🦀", "all": True, "configured": True, "layout": "cscaled"}, {"match": "title:🦀", "all": True, "configured": True, "layout": "cscaled"}),
            ("set_window_logo", {"match": None, "self": True, "alpha": .3, "position": "bottom-right"}, {"match": None, "self": True, "alpha": .3, "position": "bottom-right"}),
        ]:
            constructor = extract(checkout, f"kitty/rc/{name}.py")
            frames = [dict(p) for p in constructor(SimpleNamespace(), None, SimpleNamespace(**opts), [str(path)])]
            assert frames[-1]["data"] == "" and sum(not f["data"] for f in frames) == 1
            assert b"".join(base64.b64decode(f["data"]) for f in frames[:-1]) == body
            assert all(len(base64.b64decode(f["data"])) <= 512 for f in frames)
            identity = frames[0]["stream_id"]
            for frame in frames:
                assert frame.pop("stream_id") == identity
                frame.pop("data")
                assert frame == metadata
    constructor = extract(checkout, "kitty/rc/run.py")
    old = sys.stdin
    try:
        body = bytes(range(256)) * 33
        sys.stdin = SimpleNamespace(isatty=lambda: False, buffer=io.BytesIO(body))
        options = {"env": ["X=a=b"], "allow_remote_control": True, "remote_control_password": ["pw permit-*"]}
        frames = [dict(p) for p in constructor(SimpleNamespace(), None, SimpleNamespace(**options), ["echo", "a🦀\0binary"])]
        assert [len(base64.b64decode(f["data"])) for f in frames] == [4096, 4096, 256]
        assert b"".join(base64.b64decode(f["data"]) for f in frames) == body
        for frame in frames:
            assert frame["cmdline"] == ["echo", "a🦀\0binary"]
            for key, value in options.items(): assert frame[key] == value
    finally:
        sys.stdin = old
    print("Pinned stable upload constructors verified: repeated options, exact binary, image Python 512-byte chunks, run raw 4096-byte chunks")

if __name__ == "__main__":
    main(sys.argv[1])
