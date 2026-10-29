"""Run only inside explicit native kitty +runpy; grammar oracle only.

Rust supplies the actual action wire strings plus expected func/args over
stdin. The complete native parser includes its outer strip/split, the named
buffer remainder capture and shlex splitting via to_cmdline. This exercises
no alias dispatch, UI, clipboard or child effects.
"""
import json
import sys

from kitty.constants import str_version
from kitty.options.utils import parse_key_action

count = 0
for case in json.load(sys.stdin):
    parsed = parse_key_action(case["wire"])
    assert (parsed.func, parsed.args) == (case["func"], tuple(case["args"])), (case, parsed)
    count += 1

# Why constructors reject empty and edge-whitespace names: the bare and
# whitespace-padded shapes parse to zero arguments or an altered remainder
# rather than one verbatim buffer name.
assert parse_key_action("copy_to_buffer").args == ()
assert parse_key_action("copy_to_buffer ").args == ()
assert parse_key_action("copy_to_buffer  x").args == ("x",)
assert parse_key_action("paste_from_buffer x ").args == ("x",)
print(json.dumps({
    "kitty_version": str_version,
    "cases": count,
    "edge_shapes_checked": 4,
    "scope": "real full parser; no alias dispatch, UI or clipboard effect",
}))
