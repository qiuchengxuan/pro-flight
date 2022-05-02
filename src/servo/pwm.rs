use alloc::{boxed::Box, vec::Vec};

use embedded_hal::PwmPin;
use heapless::LinearMap;

use crate::{config::peripherals::pwm as config, datastore, fcs::out::Configuration};

type PWM = Box<dyn PwmPin<Duty = u16> + Send>;

pub struct PWMs<'a> {
    config_iteration: usize,
    pwms: Vec<(&'a str, PWM)>,
    motors: heapless::Vec<(config::Motor, usize), 4>,
    servos: LinearMap<config::ServoType, (config::Servo, usize), 4>,
}

fn to_motor_pwm_duty(max_duty: u16, rate: u16, value: u16) -> u16 {
    let duty_per_ms = max_duty as u32 * rate as u32 / 1000;
    (duty_per_ms + duty_per_ms * value as u32 / u16::MAX as u32) as u16
}

fn angle_to_axis(angle: i8) -> i32 {
    angle as i32 * i16::MAX as i32 / 90
}

fn to_servo_pwm_duty(max_duty: u16, value: i16, min: i8, max: i8, reversed: bool) -> u16 {
    let base = max_duty / 40; // 0.5ms
    let range = (max_duty / 10) as u32; // 2.0ms
    let offset = angle_to_axis((max + min) / 2);
    let mut value = if reversed { -value } else { value } as i32 + offset;
    let max = angle_to_axis(max);
    if value > max {
        value = max;
    }
    let min = angle_to_axis(min);
    if value < min {
        value = min;
    }
    let signed = value as i32 + i16::MAX as i32 + 1; // [-32767, 32767] => [1, 65535]
    base + (range * (signed as u32) / u16::MAX as u32) as u16
}

impl<'a> PWMs<'a> {
    pub fn new(pwms: Vec<(&'a str, PWM)>) -> Self {
        Self {
            config_iteration: 0,
            pwms,
            motors: heapless::Vec::new(),
            servos: heapless::LinearMap::new(),
        }
    }

    fn reconfigure(&mut self) {
        self.servos.clear();
        self.motors.clear();
        let config = crate::config::get();
        for (&id, &config) in config.peripherals.pwms.0.iter() {
            let index = match self.pwms.iter().enumerate().find(|(_, (n, _))| id.equals_str(n)) {
                Some((index, _)) => index,
                None => continue,
            };
            match config {
                config::PWM::Motor(motor) => {
                    self.motors.push((motor, index)).ok();
                }
                config::PWM::Servo(servo) => {
                    self.servos.insert(servo.servo_type, (servo, index)).ok();
                }
            }
        }
        self.motors.sort_by(|a, b| a.0.index.partial_cmp(&b.0.index).unwrap());
    }

    pub fn update(&mut self) {
        if self.config_iteration != crate::config::iteration() {
            self.reconfigure();
        }
        match datastore::acquire().read_fcs().control {
            Configuration::FixedWing(fixed_wing) => {
                for (i, &value) in fixed_wing.engines.iter().enumerate() {
                    if let Some(&(motor, index)) = self.motors.get(i) {
                        let (_, ref mut pwm) = &mut self.pwms[index];
                        let max_duty = pwm.get_max_duty();
                        pwm.set_duty(to_motor_pwm_duty(max_duty, motor.rate, value));
                    }
                }
                for &(servo_type, value) in fixed_wing.control_surface.iter() {
                    let (servo, index) = match self.servos.get(&servo_type) {
                        Some(tuple) => tuple,
                        None => continue,
                    };
                    let (_, ref mut pwm) = &mut self.pwms[*index];
                    let max_duty = pwm.get_max_duty();
                    let (min, max) = (servo.min_angle, servo.max_angle);
                    pwm.set_duty(to_servo_pwm_duty(max_duty, value, min, max, servo.reversed));
                }
            }
        }
    }
}

mod test {
    #[test]
    fn test_to_motor_pwm_duty() {
        use super::to_motor_pwm_duty;

        let max_duty = 20000;
        assert_eq!(to_motor_pwm_duty(max_duty, 400, 0), 8000);
        assert_eq!(to_motor_pwm_duty(max_duty, 400, u16::MAX / 2), 11999);
        assert_eq!(to_motor_pwm_duty(max_duty, 400, u16::MAX), 16000);
    }

    #[test]
    fn test_to_servo_pwm_duty() {
        use super::to_servo_pwm_duty;

        let max_duty = 180 * 10;
        let center = max_duty / 40 + max_duty / 20; // 0.5ms + 1.0ms
        assert_eq!(to_servo_pwm_duty(max_duty, 0, -90, 90, false), center);
        assert_eq!(to_servo_pwm_duty(max_duty, -32768, -90, 90, false), center - 90);
        assert_eq!(to_servo_pwm_duty(max_duty, 32767, -90, 90, false), center + 90);
        assert_eq!(to_servo_pwm_duty(max_duty, -8192, -90, 90, false), center - 23);
        assert_eq!(to_servo_pwm_duty(max_duty, 8192, -90, 90, false), center + 22);
    }
}
