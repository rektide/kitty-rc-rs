#!/usr/bin/env python3
"""Verify golden cases against actual methods at pinned stable revision.

Extracts selected pure constructors/parsers through AST without native kitty
imports. No server, terminal or human window I/O. Source contract verification,
not installed-server conformance. Cases correspond to Rust contracts.rs.
"""
from __future__ import annotations

import ast
import json
from pathlib import Path
import re
import subprocess
import sys
from types import SimpleNamespace

HOME = Path(__file__).resolve().parent
REVISION = "e300597364aabb4890ebdd3a5491397f858fd714"

# Explicit options/argv and independent golden values, not extracted strings
# copied from this library's serialization. Native constructors are the oracle.
COMMANDS = [
    ("action", {"self": False, "match": "id:7"}, ["goto_tab", "2"], {"action": "goto_tab 2", "self": False, "match_window": "id:7"}),
    ("close-window", {"self": False, "match": "id:8", "ignore_no_match": True}, [], {"self": False, "match": "id:8", "ignore_no_match": True}),
    ("detach-window", {"self": False, "match": "id:8", "target_tab": "new", "stay_in_tab": True}, [], {"self": False, "match": "id:8", "target_tab": "new", "stay_in_tab": True}),
    ("load-config", {"override": ["font_size=16", "background=#112233"], "ignore_overrides": True}, ["relative.conf", "/server/two.conf"], {"paths": ["relative.conf", "/server/two.conf"], "override": ["font_size=16", "background=#112233"], "ignore_overrides": True}),
    ("kitten", {"match": "id:11"}, ["custom.py", "--flag", "two words"], {"kitten": "custom.py", "args": ["--flag", "two words"], "match": "id:11"}),
    ("env", {}, ["A=b=c", "OLD"], {"env": {"A": "b=c", "OLD=": ""}}),
    ("resize-window", {"match": None, "self": True, "axis": "vertical", "increment": -3}, [], {"match": None, "self": True, "axis": "vertical", "increment": -3}),
    ("scroll-window", {"match": None}, ["0.5p-"], {"match": None, "self": True, "amount": [-0.5, "p"]}),
    ("select-window", {"match": "id:3", "self": False, "title": "pick one", "exclude_active": True, "reactivate_prev_tab": True}, [], {"match": "id:3", "self": False, "title": "pick one", "exclude_active": True, "reactivate_prev_tab": True}),
    ("send-key", {"match": None, "match_tab": None, "all": False, "exclude_active": True}, ["ctrl+a", "ctrl+b"], {"match": None, "match_tab": None, "all": False, "exclude_active": True, "keys": ["ctrl+a", "ctrl+b"]}),
    ("screenshot", {"match": "id:8", "match_tab": None}, [], {"match": "id:8", "match_tab": None, "output_path": ""}),
    ("set-user-vars", {"match": None}, ["project=a=b", "old"], {"match": None, "self": True, "var": ["project=a=b", "old"]}),
    ("set-font-size", {"all": False}, ["-2"], {"size": 2.0, "all": False, "increment_op": "-"}),
    ("set-window-title", {"match": "id:8", "temporary": False}, [], {"match": "id:8", "self": True, "temporary": False}),
    ("signal-child", {"match": None}, [], {"match": None, "self": True, "signals": ["SIGINT"]}),
]
PARSERS = [
    ("goto_tab_parse", "goto_tab", "-1", ["goto_tab", [0]]),
    ("neighboring_window", "neighboring_window", "down", ["neighboring_window", ["bottom"]]),
    ("resize_window", "resize_window", "narrower 3", ["resize_window", ["narrower", 3]]),
    ("scroll_to_prompt", "scroll_to_prompt", "-1 2", ["scroll_to_prompt", [-1, 2]]),
    ("parse_change_font_size", "change_font_size", "current /2", ["change_font_size", [False, "/", 2.0]]),
    ("clear_terminal", "clear_terminal", "to_cursor_scroll all", ["clear_terminal", ["to_cursor_scroll", False]]),
    ("simple_parse", "set_window_title", "a 'quoted' title", ["set_window_title", ["a 'quoted' title"]]),
]


def source(checkout: str, path: str) -> str:
    return subprocess.check_output(["git", "-C", checkout, "show", f"{REVISION}:{path}"], text=True)


def reject(message):
    raise AssertionError(message)


def extract(text: str, name: str):
    nodes = [n for n in ast.walk(ast.parse(text)) if isinstance(n, ast.FunctionDef) and n.name == name]
    assert len(nodes) == 1, (name, len(nodes))
    node = nodes[0]
    node.decorator_list = []
    module = ast.Module(body=[ast.ImportFrom(module="__future__", names=[ast.alias(name="annotations")], level=0), node], type_ignores=[])
    ast.fix_missing_locations(module)
    scope = {"log_error": reject}
    exec(compile(module, "<pinned upstream method>", "exec"), scope)
    return scope[name]


def main(checkout: str) -> None:
    files = subprocess.check_output(["git", "-C", checkout, "ls-tree", "--name-only", f"{REVISION}:kitty/rc"], text=True).splitlines()
    native = {p[:-3].replace("_", "-") for p in files if p.endswith(".py") and p not in {"base.py", "__init__.py"}}
    catalog = (HOME.parents[2] / "protocol/commands/src/catalog.rs").read_text()
    modeled = {a or b for a, b in re.findall(r'name:\s*"([^"]+)"|stable!\(\s*"([^"]+)"', catalog)}
    assert native == modeled - {"resize-window-edge"}, (native - modeled, modeled - native)
    assert len(native) == 41
    for name, opts, args, expected in COMMANDS:
        method = extract(source(checkout, f'kitty/rc/{name.replace("-", "_")}.py'), "message_to_kitty")
        actual = method(SimpleNamespace(fatal=reject), SimpleNamespace(), SimpleNamespace(**opts), args)
        assert json.loads(json.dumps(actual)) == expected, (name, actual, expected)
    parsers = source(checkout, "kitty/options/utils.py")
    for parser, action, rest, expected in PARSERS:
        actual = extract(parsers, parser)(action, rest)
        assert json.loads(json.dumps(actual)) == expected, (parser, actual, expected)
    print(f"Pinned source verified: {len(native)} commands, {len(COMMANDS)} constructor cases, {len(PARSERS)} nested parsers")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: verify-source0.sol61m.py /path/to/kovidgoyal/kitty")
    main(sys.argv[1])
