use alloc::boxed::Box;
use alloc::rc::Rc;

use ascii_osd_hud::telemetry as hud;
use nalgebra::{Quaternion, UnitQuaternion};

use crate::components::schedule::{Hertz, Schedulable};
use crate::config;
use crate::datastructures::coordinate::{Position, SphericalCoordinate};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::gnss::FixType;
use crate::datastructures::input::{ControlInput, Receiver};
use crate::datastructures::measurement::battery::Battery;
use crate::datastructures::measurement::distance::{Distance, NauticalMile};
use crate::datastructures::measurement::euler::{Euler, DEGREE_PER_DAG};
use crate::datastructures::measurement::{Acceleration, Altitude, Gyro, Velocity};
use crate::datastructures::waypoint::Steerpoint;

#[derive(Debug, Default, Copy, Clone, Value)]
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
        let rho = (self.rho * 10).to_unit(NauticalMile);
        hud::SphericalCoordinate { rho: rho.value() as u16, theta: self.theta, phi: self.phi }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RawData {
    pub acceleration: Acceleration,
    pub gyro: Gyro,
    pub quaternion: UnitQuaternion<f32>,
    pub fix_type: Option<FixType>,
}

pub struct Quat([f32; 4]);

impl From<UnitQuaternion<f32>> for Quat {
    fn from(quat: UnitQuaternion<f32>) -> Self {
        Self([quat[0], quat[1], quat[2], quat[3]])
    }
}

impl sval::value::Value for Quat {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.seq_begin(Some(4))?;
        for q in self.0.iter() {
            stream.seq_elem(q)?;
        }
        stream.seq_end()
    }
}

impl Default for RawData {
    fn default() -> Self {
        Self {
            quaternion: UnitQuaternion::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)),
            acceleration: Acceleration::default(),
            gyro: Gyro::default(),
            fix_type: None,
        }
    }
}

impl sval::value::Value for RawData {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(3 + self.fix_type.is_some() as usize))?;
        stream.map_key("acceleration")?;
        stream.map_value(&self.acceleration)?;
        stream.map_key("gyro")?;
        stream.map_value(&self.gyro)?;
        stream.map_key("quaternion")?;
        let quat: Quat = self.quaternion.into();
        stream.map_value(quat)?;
        if let Some(fix_type) = self.fix_type {
            stream.map_key("gnss-fix-type")?;
            stream.map_value(fix_type)?;
        }
        stream.map_end()
    }
}

#[derive(Copy, Clone, Default, Value, Debug)]
pub struct TelemetryData {
    pub altitude: Altitude,
    pub attitude: Attitude,
    pub heading: u16,
    pub height: Altitude,
    pub g_force: u8,
    pub velocity: Velocity<i16>,

    pub receiver: Receiver,
    pub input: ControlInput,

    pub position: Position,
    pub steerpoint: Steerpoint,

    pub battery: Battery,

    pub raw: RawData,
}

impl core::fmt::Display for TelemetryData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

pub struct TelemetryUnit<A, B, C, G, IMU, NAV> {
    altimeter: A,
    battery: B,
    accelerometer: C,
    gyroscope: G,
    imu: IMU,
    navigation: NAV,

    receiver: Option<Box<dyn DataSource<Receiver>>>,
    control_input: Option<Box<dyn DataSource<ControlInput>>>,
    gnss: Option<Box<dyn DataSource<FixType>>>,

    initial_altitude: Altitude,
    battery_cells: u8,
    telemetry: Rc<SingularData<TelemetryData>>,
}

impl<A, B, C, G, IMU, NAV> Schedulable for TelemetryUnit<A, B, C, G, IMU, NAV>
where
    A: DataSource<(Altitude, Velocity<i16>)>,
    B: DataSource<Battery>,
    C: DataSource<Acceleration>,
    G: DataSource<Gyro>,
    IMU: DataSource<UnitQuaternion<f32>>,
    NAV: DataSource<(Position, Steerpoint)>,
{
    fn schedule(&mut self) -> bool {
        let (altitude, velocity) = self.altimeter.read_last_unchecked();
        if self.initial_altitude == Distance::default() {
            self.initial_altitude = altitude;
        }
        let battery = self.battery.read_last_unchecked();
        if self.battery_cells == 0 {
            self.battery_cells = core::cmp::min(battery.0 / 4200 + 1, 8) as u8;
        }

        let quaternion = self.imu.read_last_unchecked();
        let euler: Euler = quaternion.into();
        let euler = euler * DEGREE_PER_DAG;
        let (position, steerpoint) = self.navigation.read_last_unchecked();
        let input_option = self.control_input.as_ref().map(|i| i.read_last_unchecked());
        let heading = (-euler.psi) as isize;

        let acceleration = self.accelerometer.read_last_unchecked();
        let gyro = self.gyroscope.read_last_unchecked();
        let fix_type = self.gnss.as_mut().map(|g| g.read_last_unchecked());

        let data = TelemetryData {
            attitude: euler.into(),
            altitude,
            heading: if heading > 0 { heading } else { 360 + heading } as u16,
            height: altitude - self.initial_altitude,
            velocity,
            g_force: acceleration.g_force(),
            battery: battery / self.battery_cells as u16,
            position,
            steerpoint,
            receiver: self.receiver.as_ref().map(|r| r.read_last_unchecked()).unwrap_or_default(),
            input: input_option.unwrap_or_default(),
            raw: RawData { acceleration, gyro, quaternion, fix_type },
        };
        self.telemetry.write(data);
        true
    }

    fn rate(&self) -> Hertz {
        50
    }
}

impl<A, B, C, G, IMU, NAV> TelemetryUnit<A, B, C, G, IMU, NAV> {
    pub fn new(
        altimeter: A,
        battery: B,
        accelerometer: C,
        gyroscope: G,
        imu: IMU,
        navigation: NAV,
    ) -> Self {
        let config = config::get();
        Self {
            altimeter,
            battery,
            accelerometer,
            gyroscope,
            imu,
            navigation,

            receiver: None,
            control_input: None,
            gnss: None,

            initial_altitude: Default::default(),
            battery_cells: config.battery.cells,
            telemetry: Rc::new(SingularData::default()),
        }
    }

    pub fn set_receiver(&mut self, receiver: Box<dyn DataSource<Receiver>>) {
        self.receiver = Some(receiver)
    }

    pub fn set_control_input(&mut self, input: Box<dyn DataSource<ControlInput>>) {
        self.control_input = Some(input)
    }

    pub fn set_gnss(&mut self, gnss: Box<dyn DataSource<FixType>>) {
        self.gnss = Some(gnss)
    }

    pub fn as_data_source(&self) -> impl DataSource<TelemetryData> {
        SingularDataSource::new(&self.telemetry)
    }
}
