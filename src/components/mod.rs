#[macro_use]
pub mod logger;

#[macro_use]
pub mod console;
pub mod altimeter;
pub mod ascii_hud;
pub mod cmdlet;
pub mod flight_control;
pub mod imu;
pub mod monitor;
pub mod panic;
pub mod sysled;
pub mod telemetry;

use crate::datastructures::U16DataReader;
use crate::hal::sensors::Battery;

pub use altimeter::Altimeter;
pub use imu::IMU;
pub use sysled::Sysled;
pub use telemetry::TelemetryUnit;

pub type BatterySource<'a> = U16DataReader<'a, Battery>;
