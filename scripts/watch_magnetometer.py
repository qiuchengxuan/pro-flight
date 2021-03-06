#!/usr/bin/env python3
import os
import sys
import time
from pathlib import Path

import numpy
from cli import CLI
from telemetry import get_bias, get_sensitive, read_sensor


def main():
    if len(sys.argv) < 2:
        print("Serial not specified")
        return

    path = sys.argv[1]
    if not Path(path).exists():
        print("Not found:", sys.argv[1])
        return

    cli = CLI(path)
    bias = get_bias(cli, 'magnetometer')
    sensitive = get_sensitive(cli, 'magnetism')
    while True:
        raw = read_sensor(cli, 'magnetism')
        calibrated = raw - bias
        vector = raw / sensitive
        normalized = vector / numpy.linalg.norm(vector)
        os.system('clear')
        print('raw %s' % raw)
        print('calibrated %s' % calibrated)
        print('normalized %s' % normalized)
        time.sleep(0.02)
    cli.close()

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        pass
