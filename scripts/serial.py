#!/usr/bin/env python3
import sys
sys.path.pop(0)
from serial import Serial

from pexpect_serial import SerialSpawn

def main():
    with Serial(sys.argv[1], 115200, timeout=0) as s:
        console = SerialSpawn(s)
        console.send('\r')
        console.expect('#')
        for line in sys.stdin:
            line = line.strip() + '\r'
            console.send(line)
            console.expect(line)
            console.expect('#')
            print(console.before.decode('utf-8').strip().replace('\r\n', '\n'))

if __name__ == '__main__':
    main()
