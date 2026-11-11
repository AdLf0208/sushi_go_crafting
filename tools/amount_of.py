#!/usr/bin/env python3
"""Read a stack's `sushigocrafting:amount` out of `ctl inventory`'s components SNBT.

`ctl inventory` prints a stack's components as SNBT, e.g.
`{sushigocrafting:amount:123}` or `{"sushigocrafting:amount":123}` (SNBT quotes a
key that holds a colon). Prints the number, or nothing when the stack has none.
"""
import re
import sys

PATTERN = re.compile(r'"?(?:sushigocrafting:amount)"?\s*:\s*(-?\d+)')


def main():
    match = PATTERN.search(" ".join(sys.argv[1:]))
    if match:
        print(match.group(1))
    return 0 if match else 1


if __name__ == "__main__":
    raise SystemExit(main())
