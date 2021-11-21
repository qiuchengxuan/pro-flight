use pid::Pid;

use crate::{
    datastructures::{control::Control, measurement::Gyro},
    service::info::{bulletin::BulletinReader, Reader},
};

pub struct PIDs<'a> {
    gyroscope: BulletinReader<'a, Gyro>,
    roll: Pid<f32>,
    pitch: Pid<f32>,
    yaw: Pid<f32>,
    max_rates: (f32, f32, f32),
}

impl<'a> PIDs<'a> {
    fn config_to_pid(config: &crate::config::pid::PID) -> Pid<f32> {
        let (kp, ki, kd) = (config.kp.into(), config.ki.into(), config.kd.into());
        Pid::new(kp, ki, kd, 100.0, 100.0, 100.0, 100.0, 0.0)
    }

    pub fn new(gyroscope: BulletinReader<'a, Gyro>, config: &crate::config::PIDs) -> Self {
        let roll = Self::config_to_pid(&config.roll);
        let pitch = Self::config_to_pid(&config.pitch);
        let yaw = Self::config_to_pid(&config.yaw);
        let max_rates =
            (config.roll.max_rate as f32, config.pitch.max_rate as f32, config.yaw.max_rate as f32);
        Self { gyroscope, roll, pitch, yaw, max_rates }
    }

    pub fn reconfigure(&mut self, config: &crate::config::PIDs) {
        *self = Self::new(self.gyroscope, config);
    }

    pub fn next_control(&mut self, mut control: Control) -> Control {
        self.roll.setpoint = control.roll as f32 * self.max_rates.0 / i16::MAX as f32;
        self.pitch.setpoint = control.pitch as f32 * self.max_rates.1 / i16::MAX as f32;
        self.yaw.setpoint = control.yaw as f32 * self.max_rates.2 / i16::MAX as f32;

        let (roll, pitch, yaw) = self.gyroscope.get_last().unwrap_or_default().into();
        let output = self.roll.next_control_output(roll).output;
        control.roll = (output * i16::MAX as f32 / 100.0) as i16;
        let output = self.pitch.next_control_output(pitch).output;
        control.pitch = (output * i16::MAX as f32 / 100.0) as i16;
        let output = self.yaw.next_control_output(yaw).output;
        control.yaw = (output * i16::MAX as f32 / 100.0) as i16;
        control
    }
}
