use pid::Pid;

use crate::types::{
    control::Axes,
    measurement::{unit::DEGs, Gyro},
};

pub struct PIDs {
    roll: Pid<f32>,
    pitch: Pid<f32>,
    yaw: Pid<f32>,
    max_rates: (f32, f32, f32),
}

fn ratio(axis: i16) -> f32 {
    axis as f32 / i16::MAX as f32
}

impl PIDs {
    fn config_to_pid(config: &crate::config::fcs::PID) -> Pid<f32> {
        let (kp, ki, kd) = (config.kp.into(), config.ki.into(), config.kd.into());
        Pid::new(kp, ki, kd, 1.0, 1.0, 1.0, 1.0, 0.0)
    }

    pub fn new(config: &crate::config::fcs::PIDs) -> Self {
        let roll = Self::config_to_pid(&config.roll);
        let pitch = Self::config_to_pid(&config.pitch);
        let yaw = Self::config_to_pid(&config.yaw);
        let max_rates =
            (config.roll.max_rate as f32, config.pitch.max_rate as f32, config.yaw.max_rate as f32);
        Self { roll, pitch, yaw, max_rates }
    }

    pub fn reconfigure(&mut self, config: &crate::config::fcs::PIDs) {
        *self = Self::new(config);
    }

    pub fn next_control(&mut self, axes: Axes, gyro: Gyro<DEGs>) -> Axes {
        self.roll.setpoint = ratio(axes.roll) * self.max_rates.0;
        self.pitch.setpoint = ratio(axes.pitch) * self.max_rates.1;
        self.yaw.setpoint = ratio(axes.yaw) * self.max_rates.2;

        let roll = self.roll.next_control_output(gyro.0.y().raw).output;
        let pitch = -self.pitch.next_control_output(gyro.0.x().raw).output;
        let yaw = self.yaw.next_control_output(gyro.0.z().raw).output;
        Axes {
            throttle: axes.throttle,
            roll: (roll * i16::MAX as f32) as i16,
            pitch: (pitch * i16::MAX as f32) as i16,
            yaw: (yaw * i16::MAX as f32) as i16,
        }
    }
}
