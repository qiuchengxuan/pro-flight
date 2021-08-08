import socket
from typing import Any, Dict, TypeVar

from pexpect import fdpexpect


class Session:
    def __init__(self, port: int):
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(('127.0.0.1', port))
        self._sock = sock
        self._session = fdpexpect.fdspawn(sock.fileno())
        self._session.expect('JSBSim>')
        self._attributes: Dict[str, Any] = {}
        self._set_attributes: Dict[str, Any] = {}

    def _exec(self, cmd) -> str:
        self._session.send(cmd + '\n')
        self._session.expect('JSBSim>')
        return self._session.before.decode('ascii')

    T = TypeVar('T')

    def get(self, attribute: str, type_: type) -> T:
        if attribute in self._attributes:
            return self._attributes[attribute]
        splitted = self._exec('get ' + attribute).split('=')
        if len(splitted) > 1:
            value = type_(splitted[1].strip())
            self._attributes[attribute] = value
            return value
        raise KeyError('No such attribute %s' % attribute)

    def set(self, attribute: str, value: Any):
        if attribute in self._set_attributes:
            if self._set_attributes[attribute] == value:
                return
        self._set_attributes[attribute] = value
        self._exec('set %s = %s' % (attribute, str(value)))

    def step(self):
        self._attributes.clear()
        self._exec('iterate 1')


class WithSession:
    def __init__(self, session: Session):
        self._session = session
