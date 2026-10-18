#!/usr/bin/env python3
"""Re-pin what Caelestia's scheme render leaves generic.

`caelestia scheme set` renders `~/.config/qtengine/config.json` from the
CLI's bundled template on every apply. Its fonts are the generic names
`Sans Serif` / `Monospace` — Inter and Maple Mono NF here only because
fontconfig was pointed at them, which is a coincidence the desktop should
not rely on. The icon theme is asked for in this skel's `cli.json` too, but
is pinned again here so the file is right on its own.

`hypr/hyprland/execs.lua` runs this a few seconds after the session starts,
once the shell's first scheme apply has written the file. An already-pinned
file is left untouched, so re-running costs nothing.
"""

import json
import os
import sys
from pathlib import Path

# The values this skel wants, in the shape of the file: each entry is the
# path to the value and what it should say.
PINS = [
    (("theme", "font", "family"), "Inter"),
    (("theme", "fontFixed", "family"), "Maple Mono NF"),
    (("theme", "iconTheme"), "Colloid-Dark"),
]


def read_at(node, keys):
    for key in keys:
        if not isinstance(node, dict):
            return None
        node = node.get(key)
    return node


def write_at(node, keys, value):
    *parents, leaf = keys
    for key in parents:
        child = node.get(key)
        if not isinstance(child, dict):
            child = {}
            node[key] = child
        node = child
    node[leaf] = value


def main(argv):
    path = Path(argv[1]) if len(argv) > 1 else Path.home() / ".config/qtengine/config.json"
    if not path.is_file():
        return 0  # never rendered on this machine; nothing to pin
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as err:
        print(f"{path}: cannot read ({err})", file=sys.stderr)
        return 1
    if not isinstance(data, dict):
        print(f"{path}: not a JSON object, leaving it alone", file=sys.stderr)
        return 1

    if all(read_at(data, keys) == value for keys, value in PINS):
        return 0  # already pinned; the file keeps its mtime

    for keys, value in PINS:
        write_at(data, keys, value)

    # Write beside the file and rename, so a reader never sees half a file.
    temp = path.with_name(path.name + ".new")
    temp.write_text(json.dumps(data, indent=4) + "\n", encoding="utf-8")
    os.replace(temp, path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
