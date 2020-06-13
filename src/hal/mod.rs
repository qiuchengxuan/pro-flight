pub mod io;
pub mod sensors;

use crate::datastructures::event::EventHandler;
use sensors::{Acceleration, Gyro};

pub type AccelGyroHandler = EventHandler<(Acceleration, Gyro)>;
