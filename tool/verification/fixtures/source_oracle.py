#!/usr/bin/env python3
"""Minimal executor for pinned upstream kitty source (the oracle boundary).

The Rust verification tool (tool/verification/src/source.rs) owns every case,
comparison and report. This file only extracts and executes named methods
from the upstream checkout at a pinned revision, because executing upstream
Python constructors/parsers is the only way to treat upstream as a semantic
oracle without translating its (GPL) code into this repository. It reads one
JSON request document on stdin and writes one JSON result document to stdout.

Request:
  {"checkout": "...", "revision": "...", "requests": [
     {"kind": "inventory", "path": "kitty/rc"},
     {"kind": "constructor", "path": "kitty/rc/action.py", "name": "message_to_kitty",
      "receiver": "fatal"|"empty", "second": "namespace"|"none",
      "opts": {...}, "args": [...], "stdin": "<optional base64>"},
      {"kind": "parser", "path": "kitty/options/utils.py", "name": "goto_tab_parse",
       "action": "...", "rest": "..."},
      {"kind": "send-text-handler", "payload": {...}, "bracketed_modes": [false],
       "revision": "<optional per-request override>"}]}

Response: a JSON array aligned with the requests; each element is
  {"ok": true, "value": <json value>} or {"ok": false, "error": "..."}.
"""
import ast
import base64
import io
import json
import os
import pathlib
import re
import subprocess
import sys
from types import SimpleNamespace


def read_source(checkout, revision, path):
    return subprocess.check_output(
        ["git", "-C", str(checkout), "show", f"{revision}:{path}"], text=True
    )


def reject(message):
    raise AssertionError(message)


def extract_method(text, name):
    nodes = [
        node
        for node in ast.walk(ast.parse(text))
        if isinstance(node, ast.FunctionDef) and node.name == name
    ]
    if len(nodes) != 1:
        raise AssertionError(f"expected exactly one {name}, found {len(nodes)}")
    node = nodes[0]
    node.decorator_list = []
    module = ast.Module(
        body=[
            ast.ImportFrom(
                module="__future__", names=[ast.alias(name="annotations")], level=0
            ),
            node,
        ],
        type_ignores=[],
    )
    ast.fix_missing_locations(module)
    scope = {
        "os": os,
        "sys": sys,
        "pathlib": pathlib,
        "Path": pathlib.Path,
        "base64": base64,
        "re": re,
        "standard_b64encode": base64.standard_b64encode,
        "log_error": reject,
        "is_png": lambda path: pathlib.Path(path).read_bytes().startswith(b"\x89PNG\r\n\x1a\n"),
    }
    exec(compile(module, "<pinned upstream method>", "exec"), scope)
    return scope[name]


def run_request(checkout, revision, request):
    kind = request["kind"]
    if kind == "inventory":
        listing = subprocess.check_output(
            ["git", "-C", str(checkout), "ls-tree", "--name-only", f"{revision}:{request['path']}"],
            text=True,
        )
        return listing.splitlines()
    if kind == "constructor":
        method = extract_method(read_source(checkout, revision, request["path"]), request["name"])
        receiver = SimpleNamespace(fatal=reject) if request.get("receiver") == "fatal" else SimpleNamespace()
        second = SimpleNamespace() if request.get("second", "none") == "namespace" else None
        previous_stdin = sys.stdin
        try:
            if "stdin" in request:
                body = base64.b64decode(request["stdin"])
                sys.stdin = SimpleNamespace(isatty=lambda: False, buffer=io.BytesIO(body))
            result = method(receiver, second, SimpleNamespace(**request["opts"]), request.get("args", []))
            if request.get("stream"):
                # Upload constructors yield payload frames.
                return [dict(frame) for frame in result]
            return result
        finally:
            sys.stdin = previous_stdin
    if kind == "parser":
        method = extract_method(read_source(checkout, revision, request["path"]), request["name"])
        return method(request["action"], request["rest"])
    if kind == "send-text-handler":
        # Only the window/screen/write boundary is synthetic. The handler and
        # bracketed-paste sanitizer execute unchanged from the named revision.
        # Fixture windows are already selected: this does NOT test match syntax,
        # a real child/TTY, dispatcher suppression, sessions or key events.
        payload = request["payload"]
        if payload.get("session_id") or not payload["data"].startswith(("text:", "base64:")):
            raise AssertionError("only finite text/bytes belong in this fixture")
        handler = extract_method(read_source(checkout, revision, "kitty/rc/send_text.py"), "response_from_kitty")
        handler.__globals__["WindowSystemKeyEvent"] = type("WindowSystemKeyEvent", (), {})
        handler.__globals__["sanitize_for_bracketed_paste"] = extract_method(
            read_source(checkout, revision, "kitty/utils.py"), "sanitize_for_bracketed_paste"
        )
        writes = [[] for _ in request["bracketed_modes"]]
        windows = [
            SimpleNamespace(screen=SimpleNamespace(in_bracketed_paste_mode=mode), write_to_child=output.append)
            for mode, output in zip(request["bracketed_modes"], writes)
        ]
        selection = {}

        def selected_windows(boss, window, payload_get, window_match_name):
            selection["window_match_name"] = window_match_name
            return [None, *windows]

        result = handler(
            SimpleNamespace(windows_for_payload=selected_windows),
            SimpleNamespace(active_window=windows[0]),
            None,
            payload.get,
        )
        return {"result": result, "writes": [[list(data) for data in output] for output in writes], **selection}
    raise AssertionError(f"unknown request kind: {kind}")


def main():
    document = json.loads(sys.stdin.read())
    results = []
    for request in document["requests"]:
        try:
            results.append({"ok": True, "value": run_request(document["checkout"], request.get("revision", document["revision"]), request)})
        except BaseException as error:  # boundary: report upstream failures, never crash the oracle
            results.append({"ok": False, "error": f"{type(error).__name__}: {error}"})
    json.dump(results, sys.stdout)


if __name__ == "__main__":
    main()
