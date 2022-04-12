
from .session import WithSession


class Variometer(WithSession):
    def get_vario(self) -> float:  # ft/s
        -self._session.get('velocities/v-down-fps', float)
