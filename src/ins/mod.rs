use core::time;

pub mod out;
pub mod positioning;
pub mod speedometer;
pub mod variometer;

use positioning::Positioning;
use speedometer::Speedometer;
use variometer::Variometer;

use crate::{
    config, datastore, imu,
    types::{coordinate::Position, sensor::Readout},
};

pub struct INS {
    interval: time::Duration,
    imu: imu::IMU,
    variometer: Variometer,
    speedometer: Speedometer,
    positioning: Positioning,
    initial: Option<Position>,
}

impl INS {
    pub fn new(sample_rate: usize, variometer: Variometer) -> Self {
        let interval = time::Duration::from_micros((1000_000 / sample_rate) as u64);
        let config = config::get();
        let speedometer = Speedometer::new(sample_rate);
        let positioning = Positioning::new(sample_rate);
        Self {
            interval,
            imu: imu::IMU::new(sample_rate, &config.imu),
            variometer,
            speedometer,
            positioning,
            initial: None,
        }
    }

    pub fn update(&mut self, acceleration: Readout, gyro: Readout) {
        let ds = datastore::acquire();
        let gnss = ds.read_gnss(Some(self.interval));
        let heading = gnss.map(|g| g.fixed.map(|f| f.heading)).flatten().flatten();
        let input = imu::Input { acceleration, gyro, magnetism: None, heading };
        let imu = match self.imu.update_imu(input) {
            Some(imu) => imu,
            None => return,
        };
        ds.write_imu(imu);

        let altitude = ds.read_altitude(Some(self.interval));
        let vs = altitude.map(|alt| self.variometer.update(alt));
        let vv = self.speedometer.update(imu.acceleration.0.raw, vs, gnss);
        let gnss_position = gnss.map(|g| g.fixed.map(|f| f.position)).flatten();
        if self.initial.is_none() && gnss_position.is_some() {
            self.initial = gnss_position;
        }
        self.positioning.update(vv, altitude, gnss_position);
        let p = self.positioning.position();
        let d = self.positioning.displacement();
        let ins = out::INS { velocity_vector: vv, position: p, displacement: d };
        ds.write_ins(ins);
    }

    /// Testing only
    pub fn skip_calibration(&mut self) {
        self.imu.skip_calibration()
    }
}
