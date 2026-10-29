"""Run only inside explicit native kitty +runpy; no translated escape decoder.

Rust supplies the actual action wire strings and literal expected values over
stdin. The complete native parser includes its outer strip/split and native C
escape expander. This exercises no alias dispatch, UI or child-input effects.
"""
import json
import sys

from kitty.constants import str_version
from kitty.options.utils import parse_key_action

counts = {"paste": 0, "send_text": 0}
for case in json.load(sys.stdin):
    parsed = parse_key_action(case["wire"])
    expected = (
        (case["literal"],) if case["func"] == "paste"
        else (case["modes"], case["literal"].encode("utf-8"))
    )
    assert parsed.func == case["func"] and parsed.args == expected, (case, parsed)
    counts[case["func"]] += 1

# Why constructors reject empty literals: omission does not mean one empty text
# argument. No malformed numeric escape is used to synthesize an empty argument.
assert parse_key_action("paste ").args == ()
assert parse_key_action("send_text all ").args == ("", b"")
print(json.dumps({
    "kitty_version": str_version,
    "paste_cases": counts["paste"],
    "send_text_cases": counts["send_text"],
    "empty_native_shapes_checked": 2,
    "scope": "real full parser; no alias dispatch, UI or child effect",
}))
