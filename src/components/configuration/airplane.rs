use crate::components::mixer::ControlMixer;
use crate::config;
use crate::config::output::Output;
use crate::datastructures::data_source::DataSource;
use crate::datastructures::input::ControlInput;
use crate::datastructures::schedule::Schedulable;
use crate::drivers::pwm::PwmByIdentifier;

pub struct Airplane<S, PWMS> {
    mixer: ControlMixer<S>,
    pwms: PWMS,
}

impl<S, PWMS> Airplane<S, PWMS> {
    pub fn new(mixer: ControlMixer<S>, pwms: PWMS) -> Self {
        Self { mixer, pwms }
    }
}

fn to_pwm_duty(max_duty: u16, value: i16) -> u16 {
    let unsigned = (value as i32 + i16::MAX as i32 + 1) as u16; // [-32768, 32767] => [0, 65535]
    let angle = (unsigned as u32 * 180 / u16::MAX as u32) as u16;
    max_duty / 2 + (((max_duty / 2) as u32) * angle as u32 / 180) as u16
}

impl<PWMS: PwmByIdentifier, S: DataSource<ControlInput>> Schedulable for Airplane<S, PWMS> {
    fn schedule(&mut self) {
        let input = self.mixer.mix();
        let outputs = &config::get().outputs.0;
        for &(identifier, output) in outputs.iter() {
            self.pwms.with(identifier, |pwm| {
                let max_duty = pwm.get_max_duty();
                match output {
                    Output::Motor(_, _) => {
                        let duty =
                            (max_duty as u32 * input.throttle as u32 / u16::MAX as u32) as u16;
                        pwm.set_duty(duty);
                    }
                    Output::AileronLeft => pwm.set_duty(to_pwm_duty(max_duty, input.roll)),
                    Output::AileronRight => pwm.set_duty(to_pwm_duty(max_duty, -input.roll)),
                    Output::Elevator => pwm.set_duty(to_pwm_duty(max_duty, input.pitch)),
                    Output::Rudder => pwm.set_duty(to_pwm_duty(max_duty, input.yaw)),
                    _ => (),
                }
            })
        }
    }
}

mod test {
    #[test]
    fn test_to_pwm_duty() {
        use super::to_pwm_duty;

        assert_eq!(to_pwm_duty(65535, 0), 49150);
        assert_eq!(to_pwm_duty(65535, -8192), 44963);
        assert_eq!(to_pwm_duty(65535, 8192), 53155);
        assert_eq!(to_pwm_duty(65535, -32768), 32767);
        assert_eq!(to_pwm_duty(65535, 32767), 65534);
    }
}
