use crate::components::mixer::ControlMixer;
use crate::config;
use crate::config::output::{Output, ServoType};
use crate::datastructures::data_source::DataSource;
use crate::datastructures::input::ControlInput;
use crate::datastructures::schedule::{Hertz, Schedulable};
use crate::drivers::pwm::PwmByIdentifier;

use super::pwm::{to_motor_pwm_duty, to_servo_pwm_duty};

pub struct Airplane<S, PWMS> {
    mixer: ControlMixer<S>,
    pwms: PWMS,
}

impl<S, PWMS> Airplane<S, PWMS> {
    pub fn new(mixer: ControlMixer<S>, pwms: PWMS) -> Self {
        Self { mixer, pwms }
    }
}

impl<PWMS: PwmByIdentifier, S: DataSource<ControlInput>> Schedulable for Airplane<S, PWMS> {
    fn rate(&self) -> Hertz {
        50
    }

    fn schedule(&mut self) -> bool {
        let input = self.mixer.mix();
        let outputs = &config::get().outputs.0;
        for &(identifier, output) in outputs.iter() {
            self.pwms.with(identifier, |pwm| {
                let max_duty = pwm.get_max_duty();
                let duty = match output {
                    Output::Motor(_) => to_motor_pwm_duty(max_duty, output.rate(), input.throttle),
                    Output::Servo(servo) => {
                        let axis = match servo.servo_type {
                            ServoType::AileronLeft => input.roll,
                            ServoType::AileronRight => -input.roll,
                            ServoType::Elevator => input.pitch,
                            ServoType::Rudder => input.yaw,
                        };
                        to_servo_pwm_duty(max_duty, axis, servo.center_angle, servo.reversed)
                    }
                    _ => return,
                };
                pwm.set_duty(duty);
            })
        }
        true
    }
}
