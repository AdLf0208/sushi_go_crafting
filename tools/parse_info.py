#!/usr/bin/env python3
"""Parse `/sushigocrafting board X Y Z` into shell assignments for the e2e.

The line this port prints:
  sushigocrafting:cutting_board at 202 101 0: input=sushigocrafting:cucumber x2
  clicks=3/6 next=sushigocrafting:cucumber_slices 50..199
(one line: the readout is `runtime::summary`).
"""
import re
import sys

LINE = re.compile(
    r"^(?P<block>\S+) at (?P<x>-?\d+) (?P<y>-?\d+) (?P<z>-?\d+): "
    r"input=(?P<input>\S+)(?: x(?P<count>\d+))? "
    r"clicks=(?P<clicks>\d+)/(?P<per>\d+) "
    r"next=(?P<next>\S+)(?: (?P<min>\d+)\.\.(?P<max>\d+))?$"
)


def main():
    text = " ".join(sys.argv[1:]).strip()
    match = LINE.match(text)
    if not match:
        print("ok=0")
        return 1
    fields = match.groupdict()
    print("ok=1")
    print(f"BLOCK={fields['block']}")
    print(f"INPUT={fields['input']}")
    print(f"COUNT={fields['count'] or '0'}")
    print(f"CLICKS={fields['clicks']}")
    print(f"PER={fields['per']}")
    print(f"NEXT={fields['next']}")
    print(f"AMIN={fields['min'] or '0'}")
    print(f"AMAX={fields['max'] or '0'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
