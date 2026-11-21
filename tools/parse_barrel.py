#!/usr/bin/env python3
"""Parse `/sushigocrafting barrel X Y Z` into shell assignments for the e2e.

The line this port prints:
  sushigocrafting:fermentation_barrel at 204 101 4: input=none
  fluid=minecraft:milk 1000/2000 mB progress=0/100
  next=sushigocrafting:cheese 50..249
(one line: the readout is `runtime::barrel_summary`). The tank's fluid is `none`
when it is empty, in which case its amount is 0.
"""
import re
import sys

LINE = re.compile(
    r"^(?P<block>\S+) at (?P<x>-?\d+) (?P<y>-?\d+) (?P<z>-?\d+): "
    r"input=(?P<input>\S+)(?: x(?P<count>\d+))? "
    r"fluid=(?P<fluid>\S+) (?P<mb>\d+)/(?P<cap>\d+) mB "
    r"progress=(?P<progress>\d+)/(?P<per>\d+) "
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
    print(f"ICOUNT={fields['count'] or '0'}")
    print(f"FLUID={fields['fluid']}")
    print(f"MB={fields['mb']}")
    print(f"CAP={fields['cap']}")
    print(f"PROGRESS={fields['progress']}")
    print(f"PER={fields['per']}")
    print(f"NEXT={fields['next']}")
    print(f"AMIN={fields['min'] or '0'}")
    print(f"AMAX={fields['max'] or '0'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
