#[derive(Default, Value)]
pub struct ControlInput {
    pub throttle: u16,
    pub roll: i16,
    pub pitch: i16,
    pub yaw: i16,
}

impl core::fmt::Display for ControlInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

impl ControlInput {
    pub fn scale_down(&self, percentage: u8) -> Self {
        Self {
            throttle: self.throttle,
            roll: (self.roll as u32 * 100 / percentage as u32) as i16,
            pitch: (self.pitch as u32 * 100 / percentage as u32) as i16,
            yaw: (self.yaw as u32 * 100 / percentage as u32) as i16,
        }
    }
}

impl core::ops::Add for ControlInput {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            throttle: ((self.throttle as u32 + other.throttle as u32) / 2) as u16,
            roll: self.roll + other.roll,
            pitch: self.pitch + other.pitch,
            yaw: self.yaw + other.yaw,
        }
    }
}

pub trait Controller {
    fn get_input(&self) -> ControlInput;
}

pub struct NoController;

impl Controller for NoController {
    fn get_input(&self) -> ControlInput {
        ControlInput::default()
    }
}
