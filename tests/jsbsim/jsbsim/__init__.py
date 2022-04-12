from .accelerometer import Acceleration, Accelerometer
from .altimeter import Altimeter
from .attitude import Attitude, AttitudeMeter
from .fcs import Control, FlightControlSystem
from .gyroscope import Gyro, Gyroscope
from .positioning import Position, Positioning
from .session import Session
from .speedometer import Speed, Speedometer
from .variometer import Variometer
from .velocity import Velocity, VelocityMeter


class JSBSim:
    def __init__(self, port: int):
        session = Session(port)
        self._accelerometer = Accelerometer(session)
        self._altimeter = Altimeter(session)
        self._attitudemeter = AttitudeMeter(session)
        self._fcs = FlightControlSystem(session)
        self._gyroscope = Gyroscope(session)
        self._positioning = Positioning(session)
        self._speedometer = Speedometer(session)
        self._variometer = Variometer(session)
        self._velocitymeter = VelocityMeter(session)
        self._session = session

    @property
    def acceleration(self) -> Acceleration:
        return self._accelerometer.get()

    @property
    def attitude(self) -> Attitude:
        return self._attitudemeter.get()

    @property
    def gyro(self) -> Gyro:
        return self._gyroscope.get()

    @property
    def speed(self) -> Speed:
        return self._speedometer.get()

    @property
    def position(self) -> Position:
        return self._positioning.get()

    @property
    def velocity(self) -> Velocity:
        return self._velocitymeter.get()

    @property
    def height(self) -> float:  # feet
        return self._session.get('position/h-agl-ft', float)

    @property
    def aoa(self) -> float:  # degree
        return self._session.get('aero/alpha-deg', float)

    @property
    def altitude(self) -> float:  # feet
        return self._altimeter.get_altitude()

    @property
    def vario(self) -> float:  # ft/s
        return self._variometer.get_vario()

    def step(self, control: Control):
        self._fcs.set(control)
        return self._session.step()
