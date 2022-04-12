from dataclasses import dataclass

from .session import WithSession


@dataclass
class Acceleration:
    x: float
    y: float
    z: float


class Accelerometer(WithSession):
    def get(self) -> Acceleration:
        return Acceleration(
            -self._session.get('accelerations/n-pilot-y-norm', float),
            -self._session.get('accelerations/n-pilot-x-norm', float),
            self._session.get('accelerations/n-pilot-z-norm', float)
        )
