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
            self._session.get('accelerations/Ny', float),
            self._session.get('accelerations/Nx', float),
            -self._session.get('accelerations/Nz', float)
        )
