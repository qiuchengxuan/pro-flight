use crate::components::mixer::ControlMixer;
use crate::config;
use crate::config::output::Output;
use crate::datastructures::data_source::DataSource;
use crate::datastructures::input::ControlInput;
use crate::datastructures::schedule::Schedulable;
use crate::drivers::pwm::PwmByIdentifier;

use super::servo::to_pwm_duty;

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
                    Output::Motor(_, _) => {
                        let throttle = (input.throttle as i32 - i16::MIN as i32) as u32;
                        max_duty / 2 + (max_duty as u32 / 2 * throttle / u16::MAX as u32) as u16
                    }
                    Output::AileronLeft => to_pwm_duty(max_duty, input.roll),
                    Output::AileronRight => to_pwm_duty(max_duty, -input.roll),
                    Output::Elevator => to_pwm_duty(max_duty, input.pitch),
                    Output::Rudder => to_pwm_duty(max_duty, input.yaw),
                    _ => return,
                };
                pwm.set_duty(duty);
            })
        }
    }
}
