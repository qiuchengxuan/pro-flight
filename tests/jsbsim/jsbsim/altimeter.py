from .session import WithSession


class Altimeter(WithSession):
    def get_altitude(self) -> float:  # feet
        return self._session.get('position/h-sl-ft', float)

    def get_vertical_speed(self) -> float:  # ft/s
        -self._session.get('velocities/v-down-fps', float)
