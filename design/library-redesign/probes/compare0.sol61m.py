#!/usr/bin/env python3
"""Explicit local integration experiment, not a shipping path dependency.
Usage: python3 design/library-redesign/probes/compare0.sol61m.py KUBELESS_CHECKOUT
Creates disposable test assembly under .test-agent; production Cargo is untouched.
"""
import pathlib, subprocess, sys

root = pathlib.Path.cwd()
upstream = pathlib.Path(sys.argv[1]).resolve()
scratch = root / ".test-agent" / "kubeless-comparison"
scratch.mkdir(parents=True, exist_ok=True)
(scratch / "README.md").write_text("Generated comparison-only package. Its explicit checkout dependencies are not a portable or published kitty dependency. Source probes and runner live under design/library-redesign/probes.\n")
dependencies = {
    "kubeless-control-retained-actions": "control/retained-actions",
    "kubeless-control-block-execution": "control/block-execution",
    "kubeless-control-plan": "control/plan",
    "kubeless-work-custody": "work/custody",
}
manifest = '[workspace]\n[package]\nname="kitty-kubeless-comparison"\nversion="0.0.0"\nedition="2024"\n[features]\ndefault=["host"]\nhost=[]\ntimed=[]\n[[test]]\nname="host"\npath="host.rs"\n[dependencies]\nfutures="0.3"\ntokio={version="1",features=["full","test-util"]}\n'
for name, path in dependencies.items():
    features = ', features=["host"]' if name.endswith("retained-actions") else ''
    manifest += f'{name}={{path="{upstream / path}"{features}}}\n'
(scratch / "Cargo.toml").write_text(manifest + '\n[[test]]\nname="direct"\npath="direct.rs"\n')
(scratch / "direct.rs").write_text((root / "design/library-redesign/probes/direct-journeys0.sol61m.rs").read_text())
host = (upstream / "control/retained-actions/tests/host.rs").read_text()
probe = (root / "design/library-redesign/probes/host-journeys0.sol61m.rs").read_text()
(scratch / "host.rs").write_text(host + "\n" + probe)
subprocess.run(["cargo", "test", "--manifest-path", str(scratch / "Cargo.toml")], check=True)
subprocess.run(["cargo", "test", "-p", "kubeless-work-custody", "--test", "attached"], cwd=upstream, check=True)
subprocess.run(["cargo", "+1.88.0", "test", "-p", "kitty-rc-registry", "--test", "journeys"], cwd=root, check=True)
