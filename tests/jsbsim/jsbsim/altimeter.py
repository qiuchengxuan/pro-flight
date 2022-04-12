from .session import WithSession


class Altimeter(WithSession):
    def get_altitude(self) -> float:  # feet
        return self._session.get('position/h-sl-ft', float)
