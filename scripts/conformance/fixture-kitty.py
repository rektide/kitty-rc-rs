#!/usr/bin/env python3
"""Private PATH adapter: augment ONLY the existing runner's owned fixture."""
import os
import pathlib
import sys

args = sys.argv[1:]
if args == ["--version"]:
    os.execve(os.environ["KITTY_RC_MATRIX_KITTY"], [os.environ["KITTY_RC_MATRIX_KITTY"], *args], os.environ)
index = args.index("--config") + 1
config = pathlib.Path(args[index])
# The composed runner owns this tempfile. Refuse an arbitrary user config.
if config.name != "kitty.conf" or not config.parent.name.startswith(("kitty-rc-live-", "kitty-private-upload-")):
    raise RuntimeError("refusing a non-conformance config")
root = config.parent
mode = os.environ["KITTY_RC_MATRIX_PROTECTION"]
if mode not in ("plain", "encrypted"):
    raise RuntimeError("invalid conformance protection mode")
text = config.read_text()
if mode == "plain":
    text = text.replace("allow_remote_control password", "allow_remote_control yes")
config.write_text(text + "\ndynamic_background_opacity yes\nconfirm_os_window_close 0\n")
for name in ("config", "cache", "data"):
    (root / name).mkdir(exist_ok=True)
env = dict(os.environ)
env.update(
    XDG_CONFIG_HOME=str(root / "config"),
    XDG_CACHE_HOME=str(root / "cache"),
    XDG_DATA_HOME=str(root / "data"),
)
# Read the checked-in independent fixture, not a user's installed kitten.
fixture = pathlib.Path(os.environ["KITTY_RC_MATRIX_ROOT"]) / "scripts/conformance/no-ui.py"
(root / "no-ui.py").write_text(fixture.read_text())
os.execve(os.environ["KITTY_RC_MATRIX_KITTY"], [os.environ["KITTY_RC_MATRIX_KITTY"], *args], env)
