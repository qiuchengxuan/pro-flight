use crate::hal::controller::{ControlSurfaceInput, ThrottleInput};

pub enum PWMType {
    Motor1,
    Motor2,
    Motor3,
    Motor4,
    AileronLeft,
    AileronRight,
    Elevator,
    Rudder,
}

pub trait Aircraft {
    fn control(&mut self, throttle: ThrottleInput, control: ControlSurfaceInput);
}
