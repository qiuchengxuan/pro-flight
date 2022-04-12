use core::time;

use crate::{
    datastore::DataStore,
    fcs::out::FCS,
    imu::out::IMU,
    ins::out::INS,
    protocol::serial::gnss::out::GNSS,
    types::{control::Control, measurement::voltage::Voltage},
};

#[derive(Copy, Clone, Default, Serialize)]
pub struct Collection {
    pub control: Control,
    pub fcs: FCS,
    pub gnss: Option<GNSS>,
    pub imu: IMU,
    pub ins: INS,
    pub voltage: Voltage,
}

impl core::fmt::Display for Collection {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        serde_json_core_fmt::to_fmt(f, self)
    }
}

pub struct Collector<'a>(&'a DataStore);

impl<'a> Collector<'a> {
    pub fn new(datastore: &'a DataStore) -> Self {
        Self(datastore)
    }

    pub fn collect(&self, timeout: Option<time::Duration>) -> Collection {
        Collection {
            control: self.0.read_control(timeout).unwrap_or_default(),
            fcs: self.0.read_fcs(timeout).unwrap_or_default(),
            gnss: self.0.read_gnss(timeout),
            imu: self.0.read_imu(timeout).unwrap_or_default(),
            ins: self.0.read_ins(timeout).unwrap_or_default(),
            voltage: self.0.read_voltage(timeout).unwrap_or_default(),
        }
    }
}
