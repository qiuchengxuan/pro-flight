from dataclasses import dataclass

from .session import WithSession


@dataclass
class Attitude:  # °
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
    def true_heading(self) -> float:
        return self.psi


class AttitudeMeter(WithSession):
    def get(self) -> Attitude:
        return Attitude(
            self._session.get('attitude/phi-deg', float),
            self._session.get('attitude/theta-deg', float),
            self._session.get('attitude/psi-deg', float)
        )
