from dataclasses import dataclass

from .session import WithSession


class Degree:
    def __init__(self, degree: float, direction: str):
        self._degree = degree
        self._direction = direction

    def __float__(self) -> float:
        return self._degree

    def __str__(self) -> str:
        degree = self._degree
        direction = self._direction[0]
        if degree < 0:
            degree = -degree
            direction = self._direction[1]
        deg = int(degree)
        minute = int(degree * 60) % 60
        second = int((degree * 3600) % 60 * 10)
        return '%s%02d°%02d\'%03d' % (direction, deg, minute, second)


@dataclass
class Position:
    latitude: Degree
    longitude: Degree


class Positioning(WithSession):
    def get(self) -> Position:
        return Position(
            Degree(self._session.get('position/lat-gc-deg', float), 'NS'),
            Degree(self._session.get('position/long-gc-deg', float), 'EW')
        )
