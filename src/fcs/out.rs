use heapless::Vec;
use serde::ser::SerializeMap;

use crate::{
    config::{fcs::Configuration, peripherals::pwm::ServoType},
    types::control,
};

#[derive(Clone, Debug, Default)]
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
    fn from(axes: control::Axes, configuration: Configuration) -> Self {
        let mut control_surface: Vec<(ServoType, i16), 4> = Vec::new();
        match configuration {
            Configuration::Airplane => {
                control_surface.push((ServoType::AileronLeft, -axes.roll)).ok();
                control_surface.push((ServoType::AileronRight, axes.roll)).ok();
                control_surface.push((ServoType::Elevator, axes.pitch)).ok();
                control_surface.push((ServoType::Rudder, axes.yaw)).ok();
            }
            Configuration::FlyingWing => {
                control_surface.push((ServoType::ElevonLeft, -axes.roll + axes.pitch)).ok();
                control_surface.push((ServoType::ElevonRight, axes.roll + axes.pitch)).ok();
            }
            Configuration::VTail => {
                control_surface.push((ServoType::AileronLeft, -axes.roll)).ok();
                control_surface.push((ServoType::AileronRight, axes.roll)).ok();
                let value = axes.yaw + axes.pitch;
                control_surface.push((ServoType::RuddervatorLeft, value)).ok();
                let value = -axes.yaw + axes.pitch;
                control_surface.push((ServoType::RuddervatorRight, value)).ok();
            }
        }
        Self { engines: [axes.throttle; 1], control_surface }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum FCS {
    FixedWing(FixedWing),
}

impl FCS {
    pub fn from(axes: control::Axes, configuration: Configuration) -> FCS {
        Self::FixedWing(FixedWing::from(axes, configuration))
    }
}

impl Default for FCS {
    fn default() -> Self {
        Self::FixedWing(FixedWing::default())
    }
}
