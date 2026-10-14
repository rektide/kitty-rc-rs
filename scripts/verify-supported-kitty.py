#!/usr/bin/env python3
"""Opt-in feature-gated private kitty matrix; no ambient terminal or install."""
import argparse
import hashlib
import json
import os
import pathlib
import re
import signal
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRATCH = ROOT / ".test-agent/supported-kitty"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def version_of(executable):
    text = subprocess.check_output([str(executable), "--version"], text=True).strip()
    match = re.match(r"kitty (\d+\.\d+\.\d+)\b", text)
    if not match:
        raise ValueError(f"unrecognized kitty version: {text}")
    return match[1], text


def clean_environment():
    # Especially remove all ambient kitty/key/password/discovery entrances.
    env = {k: v for k, v in os.environ.items() if not k.startswith("KITTY_")}
    for key in ("DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS"):
        env.pop(key, None)
    return env


def interrupt_fixture(process):
    # Own process group only. SIGINT lets the lifecycle owner's finally run.
    os.killpg(process.pid, signal.SIGINT)
    try:
        process.communicate(timeout=8)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kitty", action="append", type=pathlib.Path, help="explicit executable (repeat); default: installed plus acquired 0.43.1 and 0.49.2")
    parser.add_argument("--download", action="store_true", help="acquire pinned upstream 0.43.1 and 0.49.2 into ignored project scratch")
    parser.add_argument("--mode", action="append", choices=("encrypted", "plain"), help="repeat; defaults to both")
    parser.add_argument("--no-build", action="store_true", help="use prebuilt examples after a deliberate converged build")
    parser.add_argument("--finite-only", action="store_true", help="preparation/finite profiles only; omit the upload owner's conformance example")
    parser.add_argument("--cli-smoke", action="store_true", help="also exercise standalone ls/active/goto/watch with the advertised 16 MiB response budget")
    parser.add_argument("--report", type=pathlib.Path, default=SCRATCH / "matrix.json")
    args = parser.parse_args()
    SCRATCH.mkdir(parents=True, exist_ok=True)
    if args.download:
        subprocess.run([sys.executable, str(ROOT / "scripts/conformance/download-kitty.py"), "0.43.1", "0.49.2"], check=True)
    toolchain = subprocess.check_output(["mise", "exec", "--", "rustc", "--version", "--verbose"], cwd=ROOT, text=True).strip()
    selected_examples = ("supported_matrix",) if args.finite_only else ("conformance", "supported_matrix", "upload_conformance")
    if not args.no_build:
        build = ["mise", "exec", "--", "cargo", "build", "--locked", "-p", "kitty-rc"]
        if not args.finite_only:
            build.extend(["-p", "kitty-rc-registry"])
        for name in selected_examples:
            build.extend(["--example", name])
        subprocess.run(build, cwd=ROOT, check=True)
        if args.cli_smoke:
            subprocess.run(["mise", "exec", "--", "cargo", "build", "--locked", "-p", "kitty-rc-cli", "--bin", "kitty-rc"], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(["mise", "exec", "--", "cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT, text=True))
    examples = pathlib.Path(metadata["target_directory"]) / "debug/examples"
    executables = args.kitty or [pathlib.Path(shutil.which("kitty") or "/usr/bin/kitty"), *(SCRATCH / v / "app/bin/kitty" for v in ("0.43.1", "0.49.2"))]
    report = {
        "at": datetime.now(timezone.utc).isoformat(),
        "request_envelope": [0, 43, 1],
        "finite_only": args.finite_only,
        "cli_smoke": args.cli_smoke,
        "rustc": toolchain,
        "examples": {name: digest(examples / name) for name in selected_examples},
        "runner": "scripts/verify-isolated-uploads.py",
        "runner_sha256": digest(ROOT / "scripts/verify-isolated-uploads.py"),
        "servers": [],
    }
    if args.cli_smoke:
        report["cli_sha256"] = digest(examples.parent / "kitty-rc")
    failed = False
    for executable in executables:
        executable = executable.resolve()
        if not executable.exists():
            report["servers"].append({"executable": str(executable), "status": "blocked", "reason": "not acquired; run with --download or supply --kitty"})
            failed = True
            continue
        try:
            version, reported = version_of(executable)
        except (OSError, ValueError, subprocess.CalledProcessError) as error:
            report["servers"].append({"executable": str(executable), "status": "blocked", "reason": str(error)})
            failed = True
            continue
        server = {"executable": str(executable), "executable_sha256": digest(executable), "version": version, "reported_version": reported, "profiles": []}
        provenance = SCRATCH / version / "provenance.json"
        if provenance.exists() and json.loads(provenance.read_text())["executable"] == str(executable):
            server["provenance"] = json.loads(provenance.read_text())
        else:
            server["provenance"] = {"source": "explicit installed/system executable; no source revision inferred"}
        report["servers"].append(server)
        for mode in args.mode or ("encrypted", "plain"):
            print(f"Private matrix: kitty {version}, {mode}", flush=True)
            with tempfile.TemporaryDirectory(prefix="adapter-", dir=SCRATCH) as temporary:
                adapter = pathlib.Path(temporary)
                for name, script in (("kitty", "fixture-kitty.py"), ("cargo", "fixture-cargo.py")):
                    (adapter / name).symlink_to(ROOT / "scripts/conformance" / script)
                env = clean_environment()
                env.update(
                    PATH=str(adapter) + os.pathsep + env["PATH"],
                    KITTY_RC_MATRIX_KITTY=str(executable),
                    KITTY_RC_MATRIX_ROOT=str(ROOT),
                    KITTY_RC_MATRIX_PROTECTION=mode,
                    KITTY_RC_MATRIX_VERSION=version,
                    KITTY_RC_MATRIX_EXAMPLES=str(examples),
                    KITTY_RC_MATRIX_FINITE_ONLY="1" if args.finite_only else "0",
                    KITTY_RC_MATRIX_CLI="1" if args.cli_smoke else "0",
                )
                try:
                    process = subprocess.Popen([sys.executable, str(ROOT / "scripts/verify-isolated-uploads.py")], cwd=ROOT, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True)
                    output, _ = process.communicate(timeout=160)
                    output = output.replace("upload-test-secret", "<test-password>").replace("test-secret", "<test-password>")
                    status = "pass" if process.returncode == 0 else "fail"
                except subprocess.TimeoutExpired:
                    interrupt_fixture(process)
                    output = "matrix runner exceeded its bounded deadline"
                    status = "blocked"
                except KeyboardInterrupt:
                    interrupt_fixture(process)
                    raise
                records = []
                for line in output.splitlines():
                    try:
                        record = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if isinstance(record, dict) and ("case" in record or "harness" in record):
                        records.append(record)
                if not any("case" in record for record in records):
                    status = "blocked"
                server["profiles"].append({"protection": mode, "status": status, "records": records, "output": output})
                print(output, end="" if output.endswith("\n") else "\n", flush=True)
                failed |= status != "pass"
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Matrix evidence: {args.report}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
