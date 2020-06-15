use core::cell::Cell;
use core::cell::RefCell;

use ascii_osd_hud::telemetry as hud;

use crate::components::altimeter::Altimeter;
use crate::components::imu::IMU;
use crate::components::BatterySource;
use crate::config;
use crate::datastructures::measurement::Euler;
use crate::hal::sensors::Battery;

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

#[derive(Default, Value)]
pub struct TelemetryData {
    attitude: Attitude,
    altitude: i16,
    heading: u16,
    vertical_speed: i16,
    g_force: u8,
    battery: Battery,
}

impl core::fmt::Display for TelemetryData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

pub struct TelemetrySource<'a> {
    imu: IMU<'a>,
    altimeter: Altimeter<'a>,
    battery: BatterySource<'a>,
}

pub struct TelemetryUnit<'a> {
    source: RefCell<TelemetrySource<'a>>,
    initial_altitude: Cell<i16>,
    cells: Cell<u8>,
}

impl<'a> TelemetryUnit<'a> {
    pub fn get_data(&self) -> TelemetryData {
        if let Some(mut source) = self.source.try_borrow_mut().ok() {
            source.imu.update();
            source.altimeter.update();
        }
        let source = self.source.borrow();
        let imu = &source.imu;
        let altimeter = &source.altimeter;
        if self.initial_altitude.get() == 0 {
            self.initial_altitude.set(altimeter.altitude())
        }
        let battery = source.battery.read();
        if self.cells.get() == 0 {
            self.cells.set(core::cmp::min(battery.0 / 4200 + 1, 8) as u8)
        }
        let euler = imu.get_zyx_euler();
        TelemetryData {
            attitude: euler.into(),
            altitude: altimeter.altitude(),
            heading: ((-euler.psi as isize + 360) % 360) as u16,
            vertical_speed: altimeter.vertical_speed(),
            g_force: imu.g_force(),
            battery: battery / self.cells.get() as u16,
        }
    }
}

impl<'a> hud::TelemetrySource for TelemetryUnit<'a> {
    fn get_telemetry(&self) -> hud::Telemetry {
        let data = self.get_data();
        hud::Telemetry {
            altitude: data.altitude,
            attitude: data.attitude.into(),
            battery: data.battery.percentage(),
            heading: data.heading,
            g_force: data.g_force,
            height: data.altitude - self.initial_altitude.get(),
            vertical_speed: data.vertical_speed,
            ..Default::default()
        }
    }
}

impl<'a> TelemetryUnit<'a> {
    pub fn new(
        imu: IMU<'a>,
        altimeter: Altimeter<'a>,
        battery: BatterySource<'a>,
        config: &config::Battery,
    ) -> Self {
        Self {
            source: RefCell::new(TelemetrySource { imu, altimeter, battery }),
            initial_altitude: Default::default(),
            cells: Cell::new(config.cells),
        }
    }
}
