#!/usr/bin/env python3
import sys
import time
from pathlib import Path

from cli import CLI


def main():
    if len(sys.argv) < 2:
        print("Serial not specified")
        return

    path = sys.argv[1]
    if not Path(path).exists():
        print("Not found:", sys.argv[1])
        return

    interval = 1.0
    if len(sys.argv) >= 3:
        interval = float(sys.argv[2])

    repeat = 1
    if len(sys.argv) >= 4:
        repeat = int(sys.argv[3])

    cli = CLI(path)
    if repeat == -1:
        while True:
            print(cli.tx('telemetry'))
            time.sleep(interval)
    else:
        for i in range(repeat):
            print(cli.tx('telemetry'))
            time.sleep(interval)
    cli.close()

if __name__ == '__main__':
    main()
