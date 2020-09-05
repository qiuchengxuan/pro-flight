pub mod data;
pub use data::{Basic, Misc, Raw, TelemetryData};

use alloc::boxed::Box;
use alloc::rc::Rc;

use ascii_osd_hud::telemetry as hud;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::UnitQuaternion;

use crate::components::schedule::{Rate, Schedulable};
use crate::config;
use crate::datastructures::coordinate::{Position, SphericalCoordinate};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{AgingStaticData, DataWriter, StaticData};
use crate::datastructures::input::{ControlInput, Receiver};
use crate::datastructures::measurement::battery::Battery;
use crate::datastructures::measurement::euler::{Euler, DEGREE_PER_DAG};
use crate::datastructures::measurement::unit::{FTpM, Knot, Meter};
use crate::datastructures::measurement::{Acceleration, Altitude, Gyro, VelocityVector};
use crate::datastructures::waypoint::Steerpoint;
use crate::datastructures::GNSSFixed;

impl<U: Copy + Default + Into<u32>> Into<hud::SphericalCoordinate> for SphericalCoordinate<U> {
    fn into(self) -> hud::SphericalCoordinate {
        hud::SphericalCoordinate { rho: self.rho.value() as u16, theta: self.theta, phi: self.phi }
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

        let basic = Basic {
            attitude: euler.into(),
            altitude,
            heading: if heading >= 0 { heading } else { 360 + heading } as u16,
            height: altitude - self.initial_altitude,
            g_force: acceleration.g_force(),
            airspeed: vector.to_unit(Knot).distance().value() as u16,
            vario: vector.z.to_unit(FTpM).value() as i16,
        };

        let misc = Misc {
            battery: battery / self.battery_cells as u16,
            position,
            steerpoint,
            receiver: self.receiver.as_mut().map(|r| r.read(rate)).flatten().unwrap_or_default(),
            input: input_option.unwrap_or_default(),
        };

        let raw = Raw { acceleration, gyro, quaternion, gnss_fixed, speed_vector, displacement };

        let data = TelemetryData { basic, misc, raw };
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
