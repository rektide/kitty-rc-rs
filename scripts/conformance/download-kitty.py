#!/usr/bin/env python3
"""Opt-in upstream release binaries, confined to ignored project scratch."""
import argparse
import hashlib
import json
import pathlib
import subprocess
import tarfile
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRATCH = ROOT / ".test-agent/supported-kitty"
# Digests published by GitHub's release API, read 2026-10-14. These are
# integrity/provenance coordinates, not an independently verified signature.
RELEASES = {
    "0.43.1": {
        "sha256": "f2026ef8cec6fd21b58e3ae501f707c2fc922ddb9b3e79685917fc60197449fb",
        "source_revision": "7f24dd68c98d8f4712457560f25e8056cce52ec8",
    },
    "0.49.2": {
        "sha256": "d573618b911e9c461bd421b96c13c74c7f1cb2f1ac9c327818d4ff84366cf5c6",
        "source_revision": "b2cb871254fbdac19a085188813c8dfb258b95cd",
    },
}


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def acquire(version):
    release = RELEASES[version]
    directory = SCRATCH / version
    directory.mkdir(parents=True, exist_ok=True)
    archive = directory / f"kitty-{version}-x86_64.txz"
    url = f"https://github.com/kovidgoyal/kitty/releases/download/v{version}/{archive.name}"
    if not archive.exists():
        pending = archive.with_suffix(".part")
        request = urllib.request.Request(url, headers={"User-Agent": "kitty-rc-proto-rs-conformance"})
        with urllib.request.urlopen(request, timeout=60) as response, pending.open("wb") as output:
            while chunk := response.read(1024 * 1024):
                output.write(chunk)
        pending.rename(archive)
    actual = sha256(archive)
    if actual != release["sha256"]:
        raise RuntimeError(f"release digest mismatch for {archive}: {actual}")
    app = directory / "app"
    if not (app / "bin/kitty").exists():
        app.mkdir(exist_ok=True)
        with tarfile.open(archive, "r:xz") as bundle:
            # Reject escaping names/links/devices. Python >=3.12 data filter.
            bundle.extractall(app, filter="data")
    executable = app / "bin/kitty"
    reported = subprocess.check_output([str(executable), "--version"], text=True).strip()
    if not reported.startswith(f"kitty {version} "):
        raise RuntimeError(f"unexpected extracted executable version: {reported}")
    provenance = {
        "version": version,
        "reported_version": reported,
        "source_revision": release["source_revision"],
        "release_api": f"https://api.github.com/repos/kovidgoyal/kitty/releases/tags/v{version}",
        "download_url": url,
        "archive_sha256": actual,
        "executable": str(executable),
        "executable_sha256": sha256(executable),
        "signature_verified": False,
    }
    (directory / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(json.dumps(provenance))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("versions", nargs="+", choices=tuple(RELEASES))
    args = parser.parse_args()
    for version in args.versions:
        acquire(version)
