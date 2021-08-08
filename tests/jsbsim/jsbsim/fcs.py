from dataclasses import dataclass

from .session import WithSession


@dataclass
class Control:
    throttle: float
    aileron: float
    elevator: float
    rudder: float


class FlightControlSystem(WithSession):
    def set(self, control: Control):
        self._session.set('throttle-cmd-norm', control.throttle)
        self._session.set('aileron-cmd-norm', control.aileron)
        self._session.set('elevator-cmd-norm', control.elevator)
        self._session.set('rudder-cmd-norm', control.rudder)
