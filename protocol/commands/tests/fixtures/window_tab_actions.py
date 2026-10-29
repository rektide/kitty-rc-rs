"""Run only inside explicit native kitty +runpy; grammar and handler-selection oracle.

Rust supplies the actual action wire strings plus expected func/args over
stdin. The complete native parser includes its outer strip/split, the named
buffer remainder capture and shlex splitting via to_cmdline. The handler
oracle additionally calls the real unbound Boss.close_other_tabs_in_os_window
on a tiny fake self to check which tabs it SELECTS. This exercises no alias
dispatch, UI, clipboard, child, focus or close effects: close_tab only
records, and confirmation policies never run.
"""
import json
import sys

from kitty.boss import Boss
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


class FakeTabManager:
    def __init__(self, tabs):
        self.tabs = list(tabs)

    def __iter__(self):
        return iter(self.tabs)


class FakeBoss:
    def __init__(self, tm, active_tab):
        self.active_tab_manager_with_dispatch = tm
        self.active_tab = active_tab
        self.closed = []

    def close_tab(self, tab):
        self.closed.append(tab)


def selected_tabs(tabs, globally_active):
    fake = FakeBoss(FakeTabManager(tabs) if tabs is not None else None, globally_active)
    Boss.close_other_tabs_in_os_window(fake)
    return fake.closed


# The handler reads tm from active_tab_manager_with_dispatch (dispatch-aware)
# but the preserved tab from active_tab (focused-OS-window chain, no dispatch
# involvement). The dispatch window's identity never reaches the loop body,
# so an inactive target changes only WHICH manager is selected.
inside, other_a, other_b = object(), object(), object()
foreign = object()
handler_checks = 0

# Globally active tab within the selected manager: it is preserved even when
# the dispatch targeted an inactive tab of the same (focused) OS window.
assert selected_tabs([inside, other_a, other_b], inside) == [other_a, other_b]
handler_checks += 1

# Globally active tab outside the selected manager (e.g. a nonfocused OS
# window): close_tab is attempted for EVERY tab there; the native
# confirmation policies still apply before any actual close.
assert selected_tabs([inside, other_a, other_b], foreign) == [inside, other_a, other_b]
handler_checks += 1

# Singleton guard: a single-tab manager is left untouched even when the
# globally active tab lives elsewhere.
assert selected_tabs([inside], foreign) == []
handler_checks += 1

# Missing manager: no attempts at all.
assert selected_tabs(None, inside) == []
handler_checks += 1

print(json.dumps({
    "kitty_version": str_version,
    "cases": count,
    "edge_shapes_checked": 4,
    "handler_checks": handler_checks,
    "scope": "real full parser and handler selection on fake state; no alias dispatch, UI, focus or close effect",
}))
