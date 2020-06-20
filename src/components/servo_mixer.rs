use embedded_hal::PwmPin;

pub struct ServoMixer<'a> {
    servos: &'a mut [&'a mut dyn PwmPin<Duty = u16>],
}

impl<'a> ServoMixer<'a> {
    pub fn new(servos: &'a mut [&'a mut dyn PwmPin<Duty = u16>]) -> Self {
        Self { servos }
    }

    pub fn set_angle(&mut self, index: usize, angle: i8) {
        let servo = &mut self.servos[index];
        let adder = (servo.get_max_duty() as u32) * (angle as i16 + 90) as u32 / 90;
        servo.set_duty(servo.get_max_duty() / 2 + adder as u16);
    }
}
