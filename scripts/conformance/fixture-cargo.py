#!/usr/bin/env python3
"""Execute prebuilt examples inside the unchanged isolated runner lifecycle."""
import json
import os
import pathlib
import subprocess
import sys

args = sys.argv[1:]
expected = ["run", "-p", "kitty-rc", "--example", "conformance", "--"]
uploads = ["run", "-p", "kitty-rc-registry", "--example", "upload_conformance", "--"]
upload_fixture = args[:len(uploads)] == uploads and len(args) == len(uploads) + 4
if not upload_fixture and (args[:len(expected)] != expected or len(args) != len(expected) + 3):
    raise RuntimeError("isolated runner cargo interface changed; inspect before adapting")
fixture_args = args[len(expected):]
env = dict(os.environ)
env["KITTY_RC_MATRIX_FIXTURE"] = str(pathlib.Path(fixture_args[0]).parent)
env["KITTY_RC_MATRIX_MARKER"] = "private upload fixture" if upload_fixture else "isolated conformance"
examples = ["supported_matrix"]
if env["KITTY_RC_MATRIX_PROTECTION"] == "encrypted" and env.get("KITTY_RC_MATRIX_FINITE_ONLY") != "1":
    examples.insert(0, "conformance")
    if upload_fixture:
        examples.append("upload_conformance")
failed = False
for example in examples:
    selected = fixture_args if example == "upload_conformance" else fixture_args[:3]
    result = subprocess.run([str(pathlib.Path(env["KITTY_RC_MATRIX_EXAMPLES"]) / example), *selected], env=env)
    print(json.dumps({"harness": example, "status": "pass" if result.returncode == 0 else "fail", "returncode": result.returncode}), flush=True)
    failed |= result.returncode != 0
if env.get("KITTY_RC_MATRIX_CLI") == "1":
    result = subprocess.run([sys.executable, str(pathlib.Path(env["KITTY_RC_MATRIX_ROOT"]) / "scripts/conformance/cli-smoke.py"), fixture_args[0], fixture_args[2]], env=env)
    failed |= result.returncode != 0
sys.exit(1 if failed else 0)
