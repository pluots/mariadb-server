#!/usr/bin/env python3
import subprocess
import sys

print("LOOK HERE PY SCRIPT", sys.argv, file=sys.stderr)
print("LOOK HERE PY SCRIPT E", sys.argv)

subprocess.call(sys.argv[2:])
