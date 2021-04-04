pub mod aviation;
pub mod data;
pub mod sensor;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::UnitQuaternion;

use crate::datastructures::{
    coordinate::{Displacement, Position},
    input::{ControlInput, RSSI},
    measurement::{
        battery::Battery,
        euler::{Euler, DEGREE_PER_DAG},
        unit, Acceleration, Altitude, Course, Gyro, Heading, Magnetism, Velocity, VelocityVector,
    },
    GNSSFixed,
};
use crate::sync::singular::{SingularData, SingularDataSource};
use crate::sync::DataReader;

pub use aviation::Aviation;
pub use data::{FlightData, Misc, Navigation};
pub use sensor::Sensor;

macro_rules! flight_data {
    ($($names:ident : $types:ty),+) => {
        #[derive(Default)]
        pub struct FlightDataHUB {
            $(pub $names: SingularData<$types>),+
        }

        #[derive(Copy, Clone)]
        pub struct FlightDataReader<'a> {
            $(pub $names: SingularDataSource<'a, $types>),+,
        }

        impl FlightDataHUB {
            pub fn reader(&self) -> FlightDataReader {
                FlightDataReader {
                    $($names: self.$names.reader()),+,
                }
            }
        }
    }
}

flight_data! {
    altimeter: Altitude,
    vertical_speed: Velocity<f32, unit::MpS>,
    battery: Battery,
    accelerometer: Acceleration,
    gyroscope: Gyro,
    imu: UnitQuaternion<f32>,
    speedometer: VelocityVector<f32, unit::MpS>,
    positioning: Position,
    displacement: Displacement<unit::CentiMeter>,

    rssi: RSSI,
    control_input: ControlInput,
    magnetometer: Magnetism,

    gnss_fixed: GNSSFixed,
    gnss_heading: Heading,
    gnss_course: Course,
    gnss_position: Position,
    gnss_velocity: VelocityVector<i32, unit::MMpS>
}

impl<'a> FlightDataReader<'a> {
    pub fn read(&mut self) -> FlightData {
        let quaternion = self.imu.get().unwrap_or_default();
        let euler: Euler = quaternion.into();
        let euler = euler * DEGREE_PER_DAG;
        let heading = -euler.yaw as isize;
        let altitude = self.altimeter.get();
        let battery = self.battery.get().unwrap_or_default();
        let battery_cells = core::cmp::min(battery.0 / 4200 + 1, 8) as u16;
        let aviation = Aviation {
            attitude: euler.into(),
            altitude: altitude.unwrap_or_default(),
            heading: if heading >= 0 { heading } else { 360 + heading } as u16,
            ..Default::default()
        };

        let position = self.positioning.get().unwrap_or_default();
        let speed_vector = self.speedometer.get().unwrap_or_default();
        let navigation = Navigation { position, speed_vector, ..Default::default() };

        let acceleration = self.accelerometer.get().unwrap_or_default();
        let gyro = self.gyroscope.get().unwrap_or_default();
        let magnetism = self.magnetometer.get();
        let sensor = Sensor { acceleration, gyro, magnetism, ..Default::default() };

        let displacement = self.displacement.get().unwrap_or_default();
        let input = self.control_input.get().unwrap_or_default();
        let misc = Misc {
            battery: battery / battery_cells as u16,
            displacement,
            input,
            quaternion,
            ..Default::default()
        };

        FlightData { aviation, navigation, sensor, misc }
    }
}
