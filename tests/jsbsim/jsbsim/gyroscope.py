from dataclasses import dataclass

from .session import WithSession


@dataclass
class Gyro:  # rad/s²
    phi: float
    theta: float
    psi: float

    @property
    def roll(self) -> float:
        return self.phi

    @property
    def pitch(self) -> float:
        return self.theta

    @property
    def yaw(self) -> float:
        return self.psi


class Gyroscope(WithSession):
    def get(self) -> Gyro:
        return Gyro(
            self._session.get('velocities/phidot-rad_sec', float),
            self._session.get('velocities/thetadot-rad_sec', float),
            self._session.get('velocities/psidot-rad_sec', float)
        )
