from .session import WithSession


class Velocity:  # ft/s
    x: float
    y: float
    z: float


class VelocityMeter(WithSession):
    def get(self) -> Velocity:
        return Velocity(
            self._session.get('velocities/v-fps', float),
            self._session.get('velocities/u-fps', float),
            self._session.get('velocities/w-fps', float)
        )
