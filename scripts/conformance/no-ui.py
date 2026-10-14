"""Independent test-only kitten. Installed solely in the private fixture."""
import json
import pathlib
from kittens.tui.handler import result_handler


def main(args):
    raise RuntimeError("the matrix fixture must never open a kitten UI")


@result_handler(no_ui=True)
def handle_result(args, answer, target_window_id, boss):
    values = args[1:]
    if values and values[0] == "record":
        pathlib.Path(values[1]).write_text(json.dumps(values[2:]))
        return True
    if values == ["boolean"]:
        return True
    return json.dumps(values, ensure_ascii=False)
