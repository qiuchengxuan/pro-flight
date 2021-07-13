use alloc::boxed::Box;
use alloc::vec::Vec;

use embedded_hal::PwmPin;

use crate::{
    components::configuration::ControlSurface,
    components::mixer::ControlMixer,
    config,
    config::aircraft::Configuration,
    config::peripherals::pwm::{ServoType, PWM as PwmConfig},
    datastructures::input::ControlInput,
    sync::{AgingDataReader, DataReader},
};

use super::pwm::{to_motor_pwm_duty, to_servo_pwm_duty};

type PWM = Box<dyn PwmPin<Duty = u16> + Send>;

pub struct FixedWing<R, S> {
    mixer: ControlMixer<R, S>,
    pwms: Vec<(&'static str, PWM)>,
    configs: Vec<Option<PwmConfig>>,
    config_iteration: usize,
}

impl<R, S> FixedWing<R, S> {
    pub fn new(mixer: ControlMixer<R, S>, pwms: Vec<(&'static str, PWM)>) -> Self {
        let config_iteration = config::iteration();
        let configs = vec![None; pwms.len()];
        Self { mixer, pwms, configs, config_iteration }
    }

    fn reload_config(&mut self) {
        let config = &config::get().peripherals.pwms;
        for (i, (name, _)) in self.pwms.iter().enumerate() {
            self.configs[i] = config.get(name).map(|pwm| pwm.clone());
        }
    }
}

impl<R, S> ControlSurface for FixedWing<R, S>
where
    R: AgingDataReader<ControlInput> + Send,
    S: DataReader<ControlInput> + Send,
{
    fn update(&mut self) {
        if self.config_iteration != config::iteration() {
            self.reload_config();
        }
        let input = self.mixer.mix();
        let (left, right) = match config::get().aircraft.configuration {
            Configuration::FlyingWing => (-input.roll + input.pitch, input.roll + input.pitch),
            Configuration::VTail => (input.yaw + input.pitch, -input.yaw + input.pitch),
            _ => (0, 0),
        };
        for (i, (_, pwm)) in self.pwms.iter_mut().enumerate() {
            let config = match self.configs[i] {
                Some(config) => config,
                None => continue,
            };
            let max_duty = pwm.get_max_duty();
            let duty = match config {
                PwmConfig::Motor(_) => to_motor_pwm_duty(max_duty, config.rate(), input.throttle),
                PwmConfig::Servo(servo) => {
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
        }
    }
}
