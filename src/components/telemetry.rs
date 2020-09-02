use alloc::boxed::Box;
use alloc::rc::Rc;

use ascii_osd_hud::telemetry as hud;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{Quaternion, UnitQuaternion};

use crate::components::schedule::{Rate, Schedulable};
use crate::config;
use crate::datastructures::coordinate::{Position, SphericalCoordinate};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{AgingStaticData, DataWriter, StaticData};
use crate::datastructures::input::{ControlInput, Receiver};
use crate::datastructures::measurement::battery::Battery;
use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::euler::{Euler, DEGREE_PER_DAG};
use crate::datastructures::measurement::unit::{FTpM, Knot, Meter};
use crate::datastructures::measurement::{Acceleration, Altitude, Gyro, VelocityVector};
use crate::datastructures::waypoint::Steerpoint;
use crate::datastructures::GNSSFixed;

#[derive(Debug, Default, Copy, Clone)]
pub struct Attitude {
    pub roll: i16,
    pub pitch: i16,
}

impl From<Euler> for Attitude {
    fn from(euler: Euler) -> Self {
        let roll = (-euler.roll * 10.0) as i16;
        let pitch = (-euler.pitch * 10.0) as i16;
        Self { roll, pitch }
    }
}

impl Into<hud::Attitude> for Attitude {
    fn into(self) -> hud::Attitude {
        hud::Attitude { roll: self.roll / 10, pitch: (self.pitch / 10) as i8 }
    }
}

impl sval::value::Value for Attitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(2))?;
        stream.map_key("roll")?;
        stream.map_value(self.roll / 10)?;
        stream.map_key("pitch")?;
        stream.map_value(self.pitch / 10)?;
        stream.map_end()
    }
}

impl<U: Copy + Default + Into<u32>> Into<hud::SphericalCoordinate> for SphericalCoordinate<U> {
    fn into(self) -> hud::SphericalCoordinate {
        hud::SphericalCoordinate { rho: self.rho.value() as u16, theta: self.theta, phi: self.phi }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RawData {
    pub acceleration: Acceleration,
    pub gyro: Gyro,
    pub quaternion: UnitQuaternion<f32>,
    pub gnss_fixed: Option<bool>,
    pub speed_vector: VelocityVector<f32, Meter>,
    pub displacement: DistanceVector<i32, Meter>,
}

impl Default for RawData {
    fn default() -> Self {
        Self {
            quaternion: UnitQuaternion::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)),
            acceleration: Acceleration::default(),
            gyro: Gyro::default(),
            gnss_fixed: None,
            speed_vector: VelocityVector::default(),
            displacement: DistanceVector::default(),
        }
    }
}

impl sval::value::Value for RawData {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(5 + if self.gnss_fixed.is_some() { 1 } else { 0 }))?;
        stream.map_key("acceleration")?;
        stream.map_value(&self.acceleration)?;
        stream.map_key("gyro")?;
        stream.map_value(&self.gyro)?;
        stream.map_key("quaternion")?;
        let q = &self.quaternion;
        let value: [f32; 4] = [q.i, q.j, q.k, q.w];
        stream.map_value(&value[..])?;
        if let Some(fixed) = self.gnss_fixed {
            stream.map_key("gnss-fix")?;
            stream.map_value(fixed)?;
        }
        stream.map_key("speed-vector")?;
        stream.map_value(&self.speed_vector)?;
        stream.map_key("displacement")?;
        stream.map_value(&self.displacement)?;
        stream.map_end()
    }
}

#[derive(Copy, Clone, Default, Debug, Value)]
pub struct TelemetryData {
    pub altitude: Altitude,
    pub attitude: Attitude,
    pub heading: u16,
    pub height: Altitude,
    pub g_force: u8,
    pub airspeed: u16,
    pub vario: i16,

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

pub struct TelemetryUnit<A, B, C, G, IMU, S, NAV> {
    altimeter: A,
    battery: B,
    accelerometer: C,
    gyroscope: G,
    imu: IMU,
    speedometer: S,
    navigation: NAV,

    receiver: Option<Box<dyn AgingStaticData<Receiver>>>,
    control_input: Option<Box<dyn AgingStaticData<ControlInput>>>,
    gnss_fix: Option<Box<dyn StaticData<GNSSFixed>>>,

    initial_altitude: Altitude,
    battery_cells: u8,
    telemetry: Rc<SingularData<TelemetryData>>,
}

impl<A, B, ACCEL, G, IMU, S, NAV> Schedulable for TelemetryUnit<A, B, ACCEL, G, IMU, S, NAV>
where
    A: StaticData<Altitude>,
    B: StaticData<Battery>,
    ACCEL: StaticData<Acceleration>,
    G: StaticData<Gyro>,
    IMU: StaticData<UnitQuaternion<f32>>,
    S: StaticData<VelocityVector<f32, Meter>>,
    NAV: StaticData<(Position, Steerpoint)>,
{
    fn schedule(&mut self) -> bool {
        let rate = self.rate();

        let altitude = self.altimeter.read();
        if self.initial_altitude.is_zero() {
            self.initial_altitude = altitude;
        }
        let battery = self.battery.read();
        if self.battery_cells == 0 {
            self.battery_cells = core::cmp::min(battery.0 / 4200 + 1, 8) as u8;
        }

        let quaternion = self.imu.read();
        let euler: Euler = quaternion.into();
        let euler = euler * DEGREE_PER_DAG;
        let (position, steerpoint) = self.navigation.read();
        let input_option = self.control_input.as_mut().map(|i| i.read(rate)).flatten();
        let heading = -euler.yaw as isize;

        let acceleration = self.accelerometer.read();
        let gyro = self.gyroscope.read();
        let gnss_fixed = self.gnss_fix.as_mut().map(|g| g.read().into());

        let speed_vector = self.speedometer.read();
        let vector = speed_vector.convert(|v| v as i32);

        let displacement = steerpoint.waypoint.position - position;

        let data = TelemetryData {
            attitude: euler.into(),
            altitude,
            heading: if heading >= 0 { heading } else { 360 + heading } as u16,
            height: altitude - self.initial_altitude,
            g_force: acceleration.g_force(),
            airspeed: vector.to_unit(Knot).distance().value() as u16,
            vario: vector.z.to_unit(FTpM).value() as i16,
            battery: battery / self.battery_cells as u16,
            position,
            steerpoint,
            receiver: self.receiver.as_mut().map(|r| r.read(rate)).flatten().unwrap_or_default(),
            input: input_option.unwrap_or_default(),
            raw: RawData { acceleration, gyro, quaternion, gnss_fixed, speed_vector, displacement },
        };
        self.telemetry.write(data);
        true
    }

    fn rate(&self) -> Rate {
        50
    }
}

impl<A, B, C, G, IMU, S, NAV> TelemetryUnit<A, B, C, G, IMU, S, NAV> {
    pub fn new(
        altimeter: A,
        battery: B,
        accelerometer: C,
        gyroscope: G,
        imu: IMU,
        speedometer: S,
        navigation: NAV,
    ) -> Self {
        let config = config::get();
        Self {
            altimeter,
            battery,
            accelerometer,
            gyroscope,
            imu,
            speedometer,
            navigation,

            receiver: None,
            control_input: None,
            gnss_fix: None,

            initial_altitude: Default::default(),
            battery_cells: config.battery.cells,
            telemetry: Rc::new(SingularData::default()),
        }
    }

    pub fn set_receiver(&mut self, receiver: Box<dyn AgingStaticData<Receiver>>) {
        self.receiver = Some(receiver)
    }

    pub fn set_control_input(&mut self, input: Box<dyn AgingStaticData<ControlInput>>) {
        self.control_input = Some(input)
    }

    pub fn set_gnss(&mut self, gnss: Box<dyn StaticData<GNSSFixed>>) {
        self.gnss_fix = Some(gnss)
    }

    pub fn reader(&self) -> SingularDataSource<TelemetryData> {
        SingularDataSource::new(&self.telemetry)
    }
}
