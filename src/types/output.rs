use serde::ser::SerializeMap;

use crate::{
    config::{fcs::Configuration, peripherals::pwm::ServoType},
    types::{control::Control, vec::Vec},
};

#[derive(Copy, Clone, Debug, Default)]
pub struct FixedWing {
    pub engines: [u16; 1],
    pub control_surface: Vec<(ServoType, i16), 4>,
}

impl serde::Serialize for FixedWing {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1 + self.control_surface.len()))?;
        map.serialize_entry("engines", &self.engines)?;
        for (servo_type, value) in self.control_surface.iter() {
            map.serialize_entry(servo_type, value)?;
        }
        map.end()
    }
}

impl FixedWing {
    fn from(control: &Control, configuration: Configuration) -> Self {
        let mut control_surface: Vec<(ServoType, i16), 4> = Vec::new();
        match configuration {
            Configuration::Airplane => {
                control_surface.push((ServoType::AileronLeft, -control.roll)).ok();
                control_surface.push((ServoType::AileronRight, control.roll)).ok();
                control_surface.push((ServoType::Elevator, control.pitch)).ok();
                control_surface.push((ServoType::Rudder, control.yaw)).ok();
            }
            Configuration::FlyingWing => {
                control_surface.push((ServoType::ElevonLeft, -control.roll + control.pitch)).ok();
                control_surface.push((ServoType::ElevonRight, control.roll + control.pitch)).ok();
            }
            Configuration::VTail => {
                control_surface.push((ServoType::AileronLeft, -control.roll)).ok();
                control_surface.push((ServoType::AileronRight, control.roll)).ok();
                let value = control.yaw + control.pitch;
                control_surface.push((ServoType::RuddervatorLeft, value)).ok();
                let value = -control.yaw + control.pitch;
                control_surface.push((ServoType::RuddervatorRight, value)).ok();
            }
        }
        Self { engines: [control.throttle; 1], control_surface }
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Output {
    FixedWing(FixedWing),
}

impl Output {
    pub fn from(control: &Control, configuration: Configuration) -> Output {
        Self::FixedWing(FixedWing::from(control, configuration))
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::FixedWing(FixedWing::default())
    }
}
