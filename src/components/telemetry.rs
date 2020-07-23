use core::cell::Cell;

use ascii_osd_hud::telemetry as hud;
use nalgebra::UnitQuaternion;

use crate::config;
use crate::datastructures::coordinate::{Displacement, Position, SphericalCoordinate};
use crate::datastructures::data_source::DataSource;
use crate::datastructures::input::{ControlInput, Receiver};
use crate::datastructures::measurement::battery::Battery;
use crate::datastructures::measurement::euler::{Euler, DEGREE_PER_DAG};
use crate::datastructures::measurement::{
    Acceleration, Altitude, Distance, DistanceUnit, Velocity,
};
use crate::datastructures::waypoint::Steerpoint;

#[derive(Default, Copy, Clone, Value)]
pub struct Attitude {
    roll: i16,
    pitch: i8,
}

impl From<Euler> for Attitude {
    fn from(euler: Euler) -> Self {
        let roll = -euler.theta as i16;
        let mut pitch = -euler.phi as i8;
        if pitch > 90 {
            pitch = 90
        } else if pitch < -90 {
            pitch = -90
        };
        Self { roll, pitch }
    }
}

impl Into<hud::Attitude> for Attitude {
    fn into(self) -> hud::Attitude {
        hud::Attitude { pitch: self.pitch, roll: self.roll }
    }
}

impl Into<hud::SphericalCoordinate> for SphericalCoordinate {
    fn into(self) -> hud::SphericalCoordinate {
        let rho = self.rho.convert(DistanceUnit::CentiMeter, DistanceUnit::NauticalMile, 10) as u16;
        hud::SphericalCoordinate { rho, theta: self.theta, phi: self.phi }
    }
}

#[derive(Default, Value)]
pub struct TelemetryData {
    attitude: Attitude,
    altitude: Altitude,
    acceleration: Acceleration,
    control_input: ControlInput,
    heading: u16,
    velocity: Velocity,
    g_force: u8,
    battery: Battery,
    position: Position,
    receiver: Receiver,
    steerpoint: Steerpoint,
}

impl core::fmt::Display for TelemetryData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

pub struct TelemetryUnit<'a, A, B, C, IMU, NAV> {
    altimeter: A,
    battery: B,
    accelerometer: C,
    imu: IMU,
    navigation: NAV,
    receiver: Option<&'a mut dyn DataSource<Receiver>>,
    control_input: Option<&'a mut dyn DataSource<ControlInput>>,
    initial_altitude: Cell<Altitude>,
    battery_cells: Cell<u8>,
}

impl<'a, A, B, C, IMU, NAV> TelemetryUnit<'a, A, B, C, IMU, NAV>
where
    A: DataSource<(Altitude, Velocity)>,
    B: DataSource<Battery>,
    C: DataSource<Acceleration>,
    IMU: DataSource<UnitQuaternion<f32>>,
    NAV: DataSource<(Position, Steerpoint)>,
{
    pub fn get_data(&self) -> TelemetryData {
        let (altitude, velocity) = self.altimeter.read_last_unchecked();
        if self.initial_altitude.get() == Distance(0) {
            self.initial_altitude.set(altitude)
        }
        let battery = self.battery.read_last_unchecked();
        if self.battery_cells.get() == 0 {
            self.battery_cells.set(core::cmp::min(battery.0 / 4200 + 1, 8) as u8)
        }
        let euler: Euler = self.imu.read_last_unchecked().into();
        let euler = euler * DEGREE_PER_DAG;
        let (position, steerpoint) = self.navigation.read_last_unchecked();
        let acceleration = self.accelerometer.read_last_unchecked();
        let input_option = self.control_input.as_ref().map(|i| i.read_last_unchecked());
        TelemetryData {
            attitude: euler.into(),
            altitude,
            acceleration,
            heading: ((-euler.psi as isize + 360) % 360) as u16,
            velocity,
            g_force: acceleration.g_force(),
            battery: battery / self.battery_cells.get() as u16,
            position,
            steerpoint,
            receiver: self.receiver.as_ref().map(|r| r.read_last_unchecked()).unwrap_or_default(),
            control_input: input_option.unwrap_or_default(),
        }
    }
}

fn round_up(value: i16) -> i16 {
    (value + 5) / 10 * 10
}

impl<'a, A, B, C, IMU, NAV> hud::TelemetrySource for TelemetryUnit<'a, A, B, C, IMU, NAV>
where
    A: DataSource<(Altitude, Velocity)>,
    B: DataSource<Battery>,
    C: DataSource<Acceleration>,
    IMU: DataSource<UnitQuaternion<f32>>,
    NAV: DataSource<(Position, Steerpoint)>,
{
    fn get_telemetry(&self) -> hud::Telemetry {
        let data = self.get_data();
        let unit_quaternion = self.imu.read_last_unchecked();
        let delta = data.steerpoint.waypoint.position - data.position;
        let vector = unit_quaternion.inverse_transform_vector(&delta.into_f32_vector());
        let transformed: Displacement = (vector[0], vector[1], vector[2]).into();
        let coordinate: SphericalCoordinate = transformed.into();
        let steerpoint = hud::Steerpoint {
            number: data.steerpoint.index,
            name: data.steerpoint.waypoint.name,
            heading: delta.azimuth(),
            coordinate: coordinate.into(),
            unit: "NM",
        };
        let altitude = data.altitude.convert(DistanceUnit::CentiMeter, DistanceUnit::Feet, 1);
        let height = data.altitude - self.initial_altitude.get();
        hud::Telemetry {
            altitude: round_up(altitude as i16),
            attitude: data.attitude.into(),
            battery: data.battery.percentage(),
            heading: data.heading,
            g_force: data.g_force,
            height: height.convert(DistanceUnit::CentiMeter, DistanceUnit::Feet, 1) as i16,
            velocity: data.velocity / 100 * 100,
            steerpoint: steerpoint,
            ..Default::default()
        }
    }
}

impl<'a, A, B, C, IMU, NAV> TelemetryUnit<'a, A, B, C, IMU, NAV> {
    pub fn new(altimeter: A, battery: B, accelerometer: C, imu: IMU, navigation: NAV) -> Self {
        let config = config::get();
        Self {
            altimeter,
            battery,
            accelerometer,
            imu,
            navigation,
            initial_altitude: Default::default(),
            battery_cells: Cell::new(config.battery.cells),
            receiver: None,
            control_input: None,
        }
    }

    pub fn set_receiver(&mut self, receiver: &'a mut dyn DataSource<Receiver>) {
        self.receiver = Some(receiver)
    }

    pub fn set_control_input(&mut self, input: &'a mut dyn DataSource<ControlInput>) {
        self.control_input = Some(input)
    }
}
