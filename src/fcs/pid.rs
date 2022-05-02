use nalgebra::Vector3;
use pid::Pid;

use crate::types::measurement::{unit::DEGs, Gyro};

pub struct PIDs {
    roll: Pid<f32>,
    pitch: Pid<f32>,
    yaw: Pid<f32>,
}

impl PIDs {
    fn config_to_pid(config: &crate::config::fcs::PID) -> Pid<f32> {
        let (kp, ki, kd) = (config.kp.into(), config.ki.into(), config.kd.into());
        Pid::new(kp, ki, kd, 100.0, 15.0, 15.0, 100.0, 0.0)
    }

    pub fn new(config: &crate::config::fcs::PIDs) -> Self {
        let roll = Self::config_to_pid(&config.roll);
        let pitch = Self::config_to_pid(&config.pitch);
        let yaw = Self::config_to_pid(&config.yaw);
        Self { roll, pitch, yaw }
    }

    pub fn reconfigure(&mut self, config: &crate::config::fcs::PIDs) {
        *self = Self::new(config);
    }

    pub fn next_control(&mut self, control: Vector3<f32>, gyro: Gyro<DEGs>) -> Vector3<f32> {
        self.roll.setpoint = control.x;
        self.pitch.setpoint = control.y;
        self.yaw.setpoint = control.z;

        let roll = self.roll.next_control_output(gyro.0.y().raw).output / 100.0;
        let pitch = self.pitch.next_control_output(gyro.0.x().raw).output / 100.0;
        let yaw = self.yaw.next_control_output(gyro.0.z().raw).output / 100.0;
        Vector3::new(roll, pitch, yaw)
    }
}
