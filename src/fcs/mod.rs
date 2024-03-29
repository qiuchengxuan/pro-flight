pub mod envelop;
pub mod out;
pub mod pid;

use fugit::NanosDurationU64 as Duration;
use nalgebra::Vector3;

use crate::{algorithm::lpf::LPF, config::fcs::Configuration, datastore, types::control};

pub struct FCS {
    interval: Duration,

    config_iteration: usize,
    configuration: Configuration,
    max_rates: Vector3<f32>,
    gyro_lpfs: [LPF<f32>; 3],
    pids: pid::PIDs,
    envelop: envelop::Envelop,
}

fn ratio(axis: i16) -> f32 {
    axis as f32 / i16::MAX as f32
}

impl FCS {
    fn reconfigure(&mut self) {
        let config = crate::config::get();
        self.pids.reconfigure(&config.fcs.pids)
    }

    pub fn new(sample_rate: usize) -> Self {
        let config = crate::config::get().fcs;
        let max_roll = config.pids.roll.max_rate as f32;
        let max_pitch = config.pids.pitch.max_rate as f32;
        let max_yaw = config.pids.yaw.max_rate as f32;
        let lpf_freq: f32 = config.gyroscope.lpf.frequency.into();
        Self {
            interval: Duration::micros(1000_000 / sample_rate as u64),
            config_iteration: crate::config::iteration(),
            configuration: config.configuration,
            max_rates: Vector3::new(max_roll, max_pitch, max_yaw),
            gyro_lpfs: [LPF::<f32>::new(sample_rate as f32, lpf_freq); 3],
            pids: pid::PIDs::new(&crate::config::get().fcs.pids),
            envelop: envelop::Envelop::new(),
        }
    }

    pub fn update(&mut self) {
        if self.config_iteration != crate::config::iteration() {
            self.reconfigure();
        }

        let ds = datastore::acquire();
        let input = ds.read_control_within(self.interval).unwrap_or_default().axes;
        let mut axes = Vector3::new(
            ratio(input.roll) * self.max_rates.x,
            ratio(input.pitch) * self.max_rates.y,
            ratio(input.yaw) * self.max_rates.z,
        );
        let imu = ds.read_imu();
        axes = self.envelop.restrict(axes, imu.attitude, imu.acceleration.g_force());
        let mut gyro = imu.gyro;
        gyro.0.raw.iter_mut().enumerate().for_each(|(i, v)| *v = self.gyro_lpfs[i].filter(*v));
        axes = self.pids.next_control(axes, gyro);
        let mut output = control::Axes {
            throttle: input.throttle,
            roll: (axes.x * i16::MAX as f32) as i16,
            pitch: (axes.y * i16::MAX as f32) as i16,
            yaw: (axes.z * i16::MAX as f32) as i16,
        };
        if input.yaw.is_positive() != output.yaw.is_positive() || input.yaw == 0 {
            output.yaw = output.yaw.clamp(-i16::MAX / 10, i16::MAX / 10);
        }
        ds.write_fcs(out::FCS {
            output: axes,
            control: out::Configuration::from(output, self.configuration),
        });
    }
}
