use crate::{
    datastore::DataStore,
    fcs::out::FCS,
    imu::out::IMU,
    ins::out::INS,
    protocol::serial::gnss::out::GNSS,
    types::{
        control::Control,
        measurement::{voltage::Voltage, Altitude},
    },
};

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct Collection {
    pub altitude: Altitude,
    pub control: Control,
    pub fcs: FCS,
    pub gnss: GNSS,
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

    pub fn collect(&self) -> Collection {
        Collection {
            altitude: self.0.read_altitude(),
            control: self.0.read_control(),
            fcs: self.0.read_fcs(),
            gnss: self.0.read_gnss(),
            imu: self.0.read_imu(),
            ins: self.0.read_ins(),
            voltage: self.0.read_voltage(),
        }
    }
}
