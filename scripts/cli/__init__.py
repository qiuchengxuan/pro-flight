from pexpect_serial import SerialSpawn
from serial import Serial


class CLI:
    def __init__(self, path: str):
        self._serial = Serial(path)
        self._pexpect = SerialSpawn(self._serial)

    def close(self):
        self._pexpect.close()
        self._serial.close()

    def tx(self, cmd: str) -> str:
        self._pexpect.send('\r')
        self._pexpect.expect('cli>')
        self._pexpect.send(cmd.strip() + '\r')
        self._pexpect.expect(cmd.strip())
        self._pexpect.expect('cli>')
        return self._pexpect.before.decode('utf-8').strip().replace('\r\n', '\n')
