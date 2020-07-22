use crate::components::mixer::ControlMixer;
use crate::config;
use crate::config::output::{Output, Servo};
use crate::datastructures::data_source::DataSource;
use crate::datastructures::input::ControlInput;
use crate::datastructures::schedule::Schedulable;
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
    fn schedule(&mut self) {
        let input = self.mixer.mix();
        let outputs = &config::get().outputs.0;
        for &(identifier, output) in outputs.iter() {
            self.pwms.with(identifier, |pwm| {
                let max_duty = pwm.get_max_duty();
                let duty = match output {
                    Output::Motor(_, _) => to_motor_pwm_duty(max_duty, input.throttle),
                    Output::Servo(servo, angle) => match servo {
                        Servo::AileronLeft => to_servo_pwm_duty(max_duty, input.roll, angle),
                        Servo::AileronRight => to_servo_pwm_duty(max_duty, -input.roll, angle),
                        Servo::Elevator => to_servo_pwm_duty(max_duty, input.pitch, angle),
                        Servo::Rudder => to_servo_pwm_duty(max_duty, input.yaw, angle),
                    },
                    _ => return,
                };
                pwm.set_duty(duty);
            })
        }
    }
}
