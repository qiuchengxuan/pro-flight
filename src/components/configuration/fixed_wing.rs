use crate::components::event::OnEvent;
use crate::components::mixer::ControlMixer;
use crate::config;
use crate::config::aircraft::Configuration;
use crate::config::output::{Output, ServoType};
use crate::datastructures::data_source::StaticData;
use crate::datastructures::input::ControlInput;
use crate::drivers::pwm::PwmByIdentifier;

use super::pwm::{to_motor_pwm_duty, to_servo_pwm_duty};

pub struct FixedWing<S, PWMS> {
    mixer: ControlMixer<S>,
    pwms: PWMS,
}

impl<S, PWMS> FixedWing<S, PWMS> {
    pub fn new(mixer: ControlMixer<S>, pwms: PWMS) -> Self {
        Self { mixer, pwms }
    }
}

impl<PWMS: PwmByIdentifier, S: StaticData<ControlInput>> OnEvent for FixedWing<S, PWMS> {
    fn on_event(&mut self) {
        let input = self.mixer.mix();
        let outputs = &config::get().outputs.0;
        let (left, right) = match config::get().aircraft.configuration {
            Configuration::FlyingWing => (-input.roll + input.pitch, input.roll + input.pitch),
            Configuration::VTail => (input.yaw + input.pitch, -input.yaw + input.pitch),
            _ => (0, 0),
        };
        for (&identifier, output) in outputs.iter() {
            self.pwms.with(identifier, |pwm| {
                let max_duty = pwm.get_max_duty();
                let duty = match output {
                    Output::Motor(_) => to_motor_pwm_duty(max_duty, output.rate(), input.throttle),
                    Output::Servo(servo) => {
                        let axis = match servo.servo_type {
                            ServoType::Aileron => input.roll,
                            ServoType::Elevator => input.pitch,
                            ServoType::Rudder => input.yaw,
                            ServoType::ElevonLeft => left,
                            ServoType::ElevonRight => right,
                        };
                        let (min, max) = (servo.min_angle, servo.max_angle);
                        to_servo_pwm_duty(max_duty, axis, min, max, servo.reversed)
                    }
                };
                pwm.set_duty(duty);
            })
        }
    }
}
