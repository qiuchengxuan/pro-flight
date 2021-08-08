from dataclasses import dataclass

from .session import WithSession


@dataclass
class Position:
    latitude: float
    longitude: float


class Positioning(WithSession):
    def get(self) -> Position:
        return Position(
            self._session.get('position/lat-gc-deg', float),
            self._session.get('position/long-gc-deg', float)
        )
