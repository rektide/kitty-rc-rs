"""Pure preparation checks; no kitty instance is contacted."""
import importlib.util
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("matrix", ROOT / "scripts/verify-supported-kitty.py")
matrix = importlib.util.module_from_spec(spec)
spec.loader.exec_module(matrix)


class MatrixPreparation(unittest.TestCase):
    def test_exact_release_version(self):
        with patch.object(matrix.subprocess, "check_output", return_value="kitty 0.49.2 created by Kovid Goyal\n"):
            self.assertEqual(matrix.version_of(pathlib.Path("explicit"))[0], "0.49.2")
        with patch.object(matrix.subprocess, "check_output", return_value="kitty 0.50dev"):
            with self.assertRaises(ValueError):
                matrix.version_of(pathlib.Path("explicit"))

    def test_ambient_terminal_credentials_not_inherited(self):
        source = {"KITTY_PUBLIC_KEY":"human-key", "KITTY_RC_PASSWORD":"never-log", "KITTY_LISTEN_ON":"human.sock", "DISPLAY":":0", "WAYLAND_DISPLAY":"human-wayland", "DBUS_SESSION_BUS_ADDRESS":"human-bus", "PATH":"/usr/bin"}
        with patch.dict(os.environ, source, clear=True):
            self.assertEqual(matrix.clean_environment(), {"PATH":"/usr/bin"})

    def test_adapter_refuses_user_config_before_write(self):
        with tempfile.TemporaryDirectory(prefix="matrix-test-", dir="/tmp/opencode") as directory:
            config = pathlib.Path(directory) / "kitty.conf"
            config.write_text("human-owned fixture substitute")
            result = subprocess.run([sys.executable, str(ROOT / "scripts/conformance/fixture-kitty.py"), "--config", str(config)], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("refusing", result.stderr)
            self.assertEqual(config.read_text(), "human-owned fixture substitute")

    def test_cargo_adapter_refuses_unexpected_commands(self):
        result = subprocess.run([sys.executable, str(ROOT / "scripts/conformance/fixture-cargo.py"), "install", "anything"], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("interface changed", result.stderr)


if __name__ == "__main__":
    unittest.main()
