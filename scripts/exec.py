#!/usr/bin/python3

import argparse
import json
import time
from pathlib import Path

import jq

from cli import CLI


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--interval", help="Specify interval", type=float, default=1.0)
    parser.add_argument("-c", "--count", help="Specify count", type=int, default=1)
    parser.add_argument("--jq", help="Specify jq expression", type=str)
    parser.add_argument("serial", help="Specify serial device path", type=str)
    parser.add_argument("command", help="Specify command to execute", type=str)
    args = parser.parse_args()

    if not Path(args.serial).exists():
        print("Not found:", args.serial)
        return

    cli = CLI(args.serial)

    def tx():
        output = cli.tx(args.command)
        if args.jq is not None:
            print(jq.compile(args.jq).input(json.loads(output)).first())
        else:
            print(output)
    try:
        if args.count > 1:
            for _ in range(args.count):
                tx()
                time.sleep(args.interval)
        elif args.count <= 0:
            while True:
                tx()
                time.sleep(args.interval)
        else:
            tx()
        cli.close()
    except KeyboardInterrupt:
        cli.close()


if __name__ == '__main__':
    main()
