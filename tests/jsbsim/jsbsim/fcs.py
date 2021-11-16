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
        self._session.set('fcs/aileron-cmd-norm', control.aileron)
        self._session.set('fcs/elevator-cmd-norm', control.elevator)
        self._session.set('fcs/rudder-cmd-norm', control.rudder)
        self._session.set('fcs/throttle-cmd-norm', control.throttle)
