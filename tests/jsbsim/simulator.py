import http
import json
from dataclasses import asdict, dataclass
from urllib.parse import quote

import requests_unixsocket
from jsonpath import jsonpath

JSON_HEADER = {'Content-type': 'application/json'}


@dataclass
class Axes:
    x: float
    y: float
    z: float

    def json(self) -> str:
        x, y, z = self.x, self.y, self.z
        return '{{"x": {:.5f}, "y": {:.5f}, "z": {:.5f}}}'.format(x, y, z)


@dataclass
class Control:
    throttle: float = 0.0
    roll: float = 0.0
    pitch: float = 0.0
    yaw: float = 0.0

    def validate(self):
        assert 0.0 <= self.throttle <= 1.0
        assert -1.0 <= self.roll <= 1.0
        assert -1.0 <= self.pitch <= 1.0
        assert -1.0 <= self.yaw <= 1.0


@dataclass
class Output:
    engine: float
    aileron: float
    elevator: float
    rudder: float


class Simulator:
    def __init__(self, sock: str):
        self._session = requests_unixsocket.Session()
        self._api = 'http+unix://' + quote(sock, safe='')

    def update_input(self, input_: Control):
        api = self._api + '/input'
        input_.validate()
        body = dict(
            throttle=int(input_.throttle * 65535),
            roll=int(input_.roll * 32767),
            pitch=int(input_.pitch * 32767),
            yaw=int(input_.yaw * 32767),
        )
        response = self._session.post(api, json.dumps(body), headers=JSON_HEADER)
        assert response.status_code == http.client.OK

    def update_acceleration(self, acceleration: Axes):
        api = self._api + '/sensors/accelerometer'
        response = self._session.post(api, acceleration.json(), headers=JSON_HEADER)
        assert response.status_code == http.client.OK

    def update_gyro(self, gyro: Axes):
        api = self._api + '/sensors/gyroscope'
        response = self._session.post(api, gyro.json(), headers=JSON_HEADER)
        assert response.status_code == http.client.OK

    def update_altitude(self, altitude: float):
        api = self._api + '/sensors/altimeter'
        response = self._session.post(api, json.dumps(altitude), headers=JSON_HEADER)
        assert response.status_code == http.client.OK

    def get_output(self) -> Output:
        response = self._session.get(self._api + '/telemetry')
        assert response.status_code == http.client.OK
        output = jsonpath(response.json(), '$.output')[0]
        engine = output['engines'][0] / 65535
        aileron = output['aileron-right'] / 32767
        elevator = output['elevator'] / 32767
        rudder = output['rudder'] / 32768
        return Output(engine, aileron, elevator, rudder)
