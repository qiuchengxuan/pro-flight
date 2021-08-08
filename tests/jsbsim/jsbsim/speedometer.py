from dataclasses import dataclass

from .session import WithSession


@dataclass
class Speed:
    airspeed: float  # knot
    true_airspeed: float  # knot

    @property
    def cas(self) -> float:
        return self.airspeed

    @property
    def tas(self) -> float:
        return self.true_airspeed


class Speedometer(WithSession):
    def get(self) -> Speed:
        return Speed(
            self._session.get('velocities/vc-kts', float),
            self._session.get('velocities/vtrue-kts', float)
        )
