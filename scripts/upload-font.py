#!/usr/bin/python3

import sys
from pathlib import Path

from serial import Serial
from xmodem import XMODEM

CMD = b'osd upload-font\r\n'


def main():
    if len(sys.argv) < 3:
        print("Serial or font file not specified")
        return

    path = sys.argv[1]
    if not Path(path).exists():
        print("Not found:", sys.argv[1])
        return
    serial = Serial(path, 115200)
    serial.write(CMD)
    serial.read(len(CMD))

    path = sys.argv[2]
    if not Path(path).exists():
        return

    with open(path, 'rb') as f:
        def getc(size, timeout=0):
            return serial.read(size)

        def putc(data, timeout=0):
            return serial.write(data)

        def xmodem_callback(total_packets, _success_count, _fail_count):
            print('%d packets sent' % total_packets, end='\r')
        XMODEM(getc, putc).send(f, retry=0, timeout=1, callback=xmodem_callback)
    print('\nTransfer complete')


if __name__ == '__main__':
    main()
