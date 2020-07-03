use crate::datastructures::input::{Pitch, Roll, Throttle, Yaw};
use crate::drivers::pwm::{dummy_pwm, PWM};

use super::basic::BasicControl;

pub struct Airplane<'a> {
    motors: [&'a mut dyn PWM; 2],
    aileron_left: &'a mut dyn PWM,
    aileron_right: &'a mut dyn PWM,
    elevator: &'a mut dyn PWM,
    rudder: &'a mut dyn PWM,
}

impl<'a> Default for Airplane<'a> {
    fn default() -> Self {
        Self {
            motors: [dummy_pwm(), dummy_pwm()],
            aileron_left: dummy_pwm(),
            aileron_right: dummy_pwm(),
            elevator: dummy_pwm(),
            rudder: dummy_pwm(),
        }
    }
}

fn set_angle(servo: &mut dyn PWM, value: i16) {
    let angle = ((value as i32) * 90 / i16::MAX as i32) as i8;
    let adder = (servo.get_max_duty() as u32) * (angle + 90) as u32 / 90;
    servo.set_duty(servo.get_max_duty() / 2 + adder as u16);
}

impl<'a> Airplane<'a> {
    pub fn set_motor(&mut self, index: usize, pwm: &'a mut dyn PWM) {
        self.motors[index] = pwm;
    }

    pub fn set_aileron_left(&mut self, pwm: &'a mut dyn PWM) {
        self.aileron_left = pwm;
    }

    pub fn set_aileron_right(&mut self, pwm: &'a mut dyn PWM) {
        self.aileron_right = pwm;
    }

    pub fn set_elevator(&mut self, pwm: &'a mut dyn PWM) {
        self.elevator = pwm;
    }

    pub fn set_rudder(&mut self, pwm: &'a mut dyn PWM) {
        self.rudder = pwm;
    }
}

impl<'a> BasicControl for Airplane<'a> {
    fn set(&mut self, throttle: Throttle, roll: Roll, pitch: Pitch, yaw: Yaw) {
        let max_duty = self.motors[0].get_max_duty() as u32;
        self.motors[0].set_duty((max_duty * throttle as u32 / u16::MAX as u32) as u16);
        let max_duty = self.motors[1].get_max_duty() as u32;
        self.motors[1].set_duty((max_duty * throttle as u32 / u16::MAX as u32) as u16);

        set_angle(self.aileron_left, roll);
        set_angle(self.aileron_right, -roll);

        set_angle(self.elevator, pitch);
        set_angle(self.rudder, yaw);
    }
}
