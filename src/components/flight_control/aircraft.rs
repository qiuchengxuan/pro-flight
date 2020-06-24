use crate::hal::controller::ControlInput;

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
    fn control(&mut self, input: ControlInput);
}
