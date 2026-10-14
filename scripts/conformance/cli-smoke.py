#!/usr/bin/env python3
"""Standalone CLI journeys against ONLY the enclosing private fixture."""
import json
import os
import pathlib
import subprocess
import sys

fixture = pathlib.Path(sys.argv[1]).parent
if not fixture.name.startswith("kitty-private-upload-"):
    raise RuntimeError("refusing a non-private CLI target")
mode = os.environ["KITTY_RC_MATRIX_PROTECTION"]
binary = pathlib.Path(os.environ["KITTY_RC_MATRIX_EXAMPLES"]).parent / "kitty-rc"
common = [str(binary), "--to", sys.argv[1], "--auth", mode, "--max-response", "16777216", "--json"]
env = dict(os.environ)
if mode == "encrypted":
    # Test-only material stays in the selected source, never in a printed argv.
    common.extend(["--key-file", str(fixture / "key"), "--password-env", "MATRIX_TEST_PASSWORD"])
    env["MATRIX_TEST_PASSWORD"] = sys.argv[2]


def query(*args):
    result = subprocess.run([*common, *args], env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10)
    if result.returncode:
        raise RuntimeError(f"CLI {args[0]} failed: {result.stderr.strip()}")
    return result.stdout


try:
    rows = json.loads(query("ls"))
    if not rows:
        raise RuntimeError("CLI returned no private window")
    row = rows[0]
    active = json.loads(query("active", "--tab", str(row["tab_id"])))
    if active["id"] != row["id"]:
        raise RuntimeError("CLI explicit tab active selected a different window")
    query("goto", str(row["id"]))
    samples = [json.loads(line) for line in query("watch", "--interval-ms", "20", "--count", "2").splitlines()]
    if len(samples) != 2 or any(sample.get("active") is None and sample.get("active_status") != "no-focused" for sample in samples):
        raise RuntimeError("CLI watch did not emit two valid focus/absence samples")
    print(json.dumps({"case":"cli-standalone-list-focus-watch","status":"pass","profile":{"auth":mode,"max_response":16777216,"watch_count":2},"detail":{"listed":len(rows),"explicit_tab_active":True,"goto_ack":True,"watch_samples":len(samples)}}), flush=True)
except Exception as error:
    print(json.dumps({"case":"cli-standalone-list-focus-watch","status":"fail","detail":{"error":str(error).replace(sys.argv[2],"<test-password>")}}), flush=True)
    raise
