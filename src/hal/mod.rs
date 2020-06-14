pub mod io;
pub mod sensors;

use crate::datastructures::event::EventHandler;
use sensors::{Acceleration, Gyro, Temperature};

pub trait AccelGyroHandler = EventHandler<(Acceleration, Gyro)>;
pub trait TemperatureHandler = EventHandler<Temperature>;
