import math
from dataclasses import dataclass

from .session import WithSession


@dataclass
class Gyro:  # °/s
    phi: float  # φ
    theta: float  # θ
    psi: float  # ψ

    @property
    def x(self) -> float:
        return self.theta

    @property
    def y(self) -> float:
        return self.phi

    @property
    def z(self) -> float:
        return self.psi

    def __str__(self):
        return '{x: %.2f°/s, y: %.2f°/s, z: %.2f°/s}' % (self.x, self.y, self.z)


class Gyroscope(WithSession):
    def get(self) -> Gyro:
        return Gyro(
            self._session.get('velocities/phidot-rad_sec', float) * 180 / math.pi,
            self._session.get('velocities/thetadot-rad_sec', float) * 180 / math.pi,
            self._session.get('velocities/psidot-rad_sec', float) * 180 / math.pi
        )
