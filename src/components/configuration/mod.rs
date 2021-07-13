pub mod fixed_wing;
pub mod pwm;

use alloc::{boxed::Box, vec::Vec};

use embedded_hal::PwmPin;

use crate::{
    components::mixer::ControlMixer,
    config::{self, aircraft::Configuration},
    datastructures::input::ControlInput,
    sync::{AgingDataReader, DataReader},
};
pub use fixed_wing::FixedWing;

type PwmPins = Vec<(&'static str, Box<dyn PwmPin<Duty = u16> + Send>)>;

pub trait ControlSurface: Send {
    fn update(&mut self);
}

pub fn make_control_surface<R, S>(
    mixer: ControlMixer<R, S>,
    pwms: PwmPins,
) -> Box<dyn ControlSurface>
where
    R: AgingDataReader<ControlInput> + Send + 'static,
    S: DataReader<ControlInput> + Send + 'static,
{
    match config::get().aircraft.configuration {
        Configuration::Airplane | Configuration::FlyingWing | Configuration::VTail => {
            Box::new(FixedWing::new(mixer, pwms))
        }
    }
}
