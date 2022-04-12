from dataclasses import dataclass

from .session import WithSession


@dataclass
class Speed:
    airspeed: float  # knot
    true_airspeed: float  # knot
    ground_speed: int  # mm/s

    @property
    def cas(self) -> float:
        return self.airspeed

    @property
    def tas(self) -> float:
        return self.true_airspeed

    @property
    def gs(self) -> int:
        return self.ground_speed


class Speedometer(WithSession):
    def get(self) -> Speed:
        return Speed(
            self._session.get('velocities/vc-kts', float),
            self._session.get('velocities/vtrue-kts', float),
            int(self._session.get('velocities/vg-fps', float) / 3.3 * 10000)
        )
