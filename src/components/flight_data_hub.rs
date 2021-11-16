#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::UnitQuaternion;

use crate::{
    datastructures::{
        control::Control,
        coordinate::{Displacement, Position},
        flight::{
            aviation::Aviation,
            misc::Misc,
            navigation::Navigation,
            sensor::{Sensor, GNSS},
            FlightData,
        },
        measurement::{
            euler::{Euler, DEGREE_PER_DAG},
            unit,
            voltage::Voltage,
            Acceleration, Altitude, Course, Gyro, Heading, Magnetism, Velocity, VelocityVector,
        },
        output::Output,
        RSSI,
    },
    sync::{
        cell::{Cell, CellReader},
        DataReader,
    },
};

macro_rules! flight_data {
    ($($names:ident : $types:ty),+) => {
        #[derive(Default)]
        pub struct FlightDataHUB {
            $(pub $names: Cell<$types>),+
        }

        #[derive(Copy, Clone)]
        pub struct FlightDataReader<'a> {
            $(pub $names: CellReader<'a, $types>),+,
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
    vertical_speed: Velocity<i32, unit::CMpS>,
    voltmeter: Voltage,
    accelerometer: Acceleration,
    gyroscope: Gyro,
    imu: UnitQuaternion<f32>,
    speedometer: VelocityVector<f32, unit::MpS>,
    positioning: Position,
    displacement: Displacement<unit::CentiMeter>,

    rssi: RSSI,
    input: Control,
    magnetometer: Magnetism,

    gnss_fixed: bool,
    gnss_heading: Heading,
    gnss_course: Course,
    gnss_position: Position,
    gnss_velocity: VelocityVector<i32, unit::MMpS>,

    output: Output
}

impl<'a> FlightDataReader<'a> {
    pub fn read(&mut self) -> FlightData {
        let acceleration = self.accelerometer.get().unwrap_or_default();
        let gyro = self.gyroscope.get().unwrap_or_default();
        let magnetism = self.magnetometer.get();
        let course = self.gnss_course.get().unwrap_or_default();
        let gnss = match self.gnss_fixed.get() {
            Some(fixed) => Some(GNSS { fixed, course }),
            None => None,
        };
        let sensor = Sensor { acceleration, gyro, magnetism, gnss };

        let position = self.positioning.get().unwrap_or_default();
        let speed_vector = self.speedometer.get().unwrap_or_default();
        let navigation = Navigation { position, speed_vector, ..Default::default() };

        let voltage = self.voltmeter.get().unwrap_or_default();
        let misc = Misc {
            displacement: self.displacement.get().unwrap_or_default(),
            quaternion: self.imu.get().unwrap_or_default(),
            rssi: self.rssi.get().unwrap_or_default(),
            voltage: voltage / voltage.cells() as u16,
        };

        let euler: Euler = misc.quaternion.into();
        let euler = euler * DEGREE_PER_DAG;
        let heading = -euler.yaw as isize;
        let altitude = self.altimeter.get();
        let aviation = Aviation {
            attitude: euler.into(),
            altitude: altitude.unwrap_or_default(),
            heading: if heading >= 0 { heading } else { 360 + heading } as u16,
            height: altitude.unwrap_or_default(),
            g_force: acceleration.g_force(),
            airspeed: speed_vector.to_unit(unit::Knot).scalar().value() as u16,
            vario: self.vertical_speed.get().unwrap_or_default().to_unit(unit::FTpM).value() as i16,
        };

        let input = self.input.get().unwrap_or_default();
        let output = self.output.get().unwrap_or_default();

        FlightData { aviation, navigation, sensor, misc, input, output }
    }
}
