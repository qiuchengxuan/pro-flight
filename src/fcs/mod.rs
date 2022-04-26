pub mod out;
pub mod pid;

use fugit::NanosDurationU64 as Duration;

use crate::{config::fcs::Configuration, datastore};

pub struct FCS {
    interval: Duration,

    config_iteration: usize,
    configuration: Configuration,
    pids: pid::PIDs,
}

impl FCS {
    fn reconfigure(&mut self) {
        let config = crate::config::get();
        self.pids.reconfigure(&config.fcs.pids)
    }

    pub fn new(sample_rate: usize) -> Self {
        Self {
            interval: Duration::micros(1000_000 / sample_rate as u64),
            config_iteration: crate::config::iteration(),
            configuration: crate::config::get().fcs.configuration,
            pids: pid::PIDs::new(&crate::config::get().fcs.pids),
        }
    }

    pub fn update(&mut self) {
        if self.config_iteration != crate::config::iteration() {
            self.reconfigure();
        }

        let ds = datastore::acquire();
        let control = ds.read_control_within(self.interval).unwrap_or_default();
        let imu = ds.read_imu();
        let mut axes = self.pids.next_control(control.axes, imu.gyro);
        if control.axes.yaw.is_positive() != axes.yaw.is_positive() || control.axes.yaw == 0 {
            axes.yaw = axes.yaw.clamp(-i16::MAX / 10, i16::MAX / 10);
        }
        let output = out::FCS::from(axes, self.configuration);
        ds.write_fcs(output);
    }
}
