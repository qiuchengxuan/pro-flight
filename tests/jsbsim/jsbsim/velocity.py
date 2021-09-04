import math
from dataclasses import dataclass

from .session import WithSession


@dataclass
class Velocity:  # ft/s
    x: float
    y: float
    z: float

    def course(self) -> float:
        return math.degrees(math.atan2(self.x, self.y))


class VelocityMeter(WithSession):
    def get(self) -> Velocity:
        return Velocity(
            self._session.get('velocities/v-fps', float),
            self._session.get('velocities/u-fps', float),
            self._session.get('velocities/w-fps', float)
        )
