import http
import json
from dataclasses import asdict, dataclass
from typing import List, Union
from urllib.parse import quote

import requests_unixsocket
from dacite import from_dict

JSON_HEADER = {'Content-type': 'application/json'}


@dataclass
class Axes:
    x: float
    y: float
    z: float

    def json(self) -> str:
        x, y, z = self.x, self.y, self.z
        return '[{:.5f}, {:.5f}, {:.5f}]'.format(x, y, z)


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
class Position:
    latitude: str
    longitude: str
    altitude: int


@dataclass
class Fixed:
    position: Position
    course: float
    ground_speed: int
    velocity_vector: List[int]
    heading: float


@dataclass
class GNSS:
    fixed: Fixed


@dataclass
class Attitude:
    roll: float
    pitch: float
    yaw: float


@dataclass
class IMU:
    acceleration: List[float]
    attitude: Attitude
    gyro: List[float]


@dataclass
class FixedWing:
    engines: List[Union[int, float]]
    aileron_left: Union[int, float]
    aileron_right: Union[int, float]
    elevator: Union[int, float]
    rudder: Union[int, float]

    def normalize(self):
        engines = [e / 65536 for e in self.engines]
        array = [self.aileron_left, self.aileron_right, self.elevator, self.rudder]
        return FixedWing(engines, *[v / 32768 for v in array])


@dataclass
class FCS:
    output: List[float]
    control: FixedWing


@dataclass
class Telemtry:
    imu: IMU
    fcs: FCS


def _factory(kv) -> dict:
    return {k.replace('_', '-'): v for k, v in kv}


def _hyphen_to_underscore(dictionary: dict):
    if isinstance(dictionary, list):
        return [_hyphen_to_underscore(e) for e in dictionary]
    elif isinstance(dictionary, dict):
        return {
            key.replace('-', '_') if isinstance(key, str) else key: _hyphen_to_underscore(value)
            for (key, value) in dictionary.items()
        }
    return dictionary


class Simulator:
    def __init__(self, sock: str):
        self._session = requests_unixsocket.Session()
        self._api = 'http+unix://' + quote(sock, safe='')

    def tick(self):
        api = self._api + '/tick'
        response = self._session.post(api, timeout=1)
        assert response.status_code == http.client.OK

    def update_input(self, input_: Control):
        api = self._api + '/input'
        input_.validate()
        body = dict(
            throttle=int(input_.throttle * 65535),
            roll=int(input_.roll * 32767),
            pitch=int(input_.pitch * 32767),
            yaw=int(input_.yaw * 32767),
        )
        response = self._session.put(api, json.dumps(body), headers=JSON_HEADER, timeout=1)
        assert response.status_code == http.client.OK

    def update_acceleration(self, acceleration: Axes):
        api = self._api + '/sensors/accelerometer'
        response = self._session.put(api, acceleration.json(), headers=JSON_HEADER, timeout=1)
        assert response.status_code == http.client.OK

    def update_gyro(self, gyro: Axes):
        api = self._api + '/sensors/gyroscope'
        response = self._session.put(api, gyro.json(), headers=JSON_HEADER, timeout=1)
        assert response.status_code == http.client.OK

    def update_altitude(self, altitude: int):
        api = self._api + '/sensors/altimeter'
        response = self._session.put(api, json.dumps(altitude), headers=JSON_HEADER, timeout=1)
        assert response.status_code == http.client.OK

    def update_gnss(self, gnss: GNSS):
        api = self._api + '/sensors/gnss'
        body = json.dumps(asdict(gnss, dict_factory=_factory), ensure_ascii=False)
        response = self._session.put(api, body.encode('utf-8'), headers=JSON_HEADER, timeout=1)
        assert response.status_code == http.client.OK

    def get_telemetry(self) -> Telemtry:
        response = self._session.get(self._api + '/telemetry', timeout=1)
        assert response.status_code == http.client.OK
        return from_dict(Telemtry, _hyphen_to_underscore(response.json()))
