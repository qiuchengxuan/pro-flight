#[derive(Default, Value)]
pub struct ControlSurfaceInput {
    pub roll: i16,
    pub pitch: i16,
    pub yaw: i16,
}

impl core::fmt::Display for ControlSurfaceInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

impl ControlSurfaceInput {
    pub fn scale_down(&self, percentage: u8) -> Self {
        Self {
            roll: (self.roll as u32 * 100 / percentage as u32) as i16,
            pitch: (self.pitch as u32 * 100 / percentage as u32) as i16,
            yaw: (self.yaw as u32 * 100 / percentage as u32) as i16,
        }
    }
}

impl core::ops::Add for ControlSurfaceInput {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            roll: self.roll + other.roll,
            pitch: self.pitch + other.pitch,
            yaw: self.yaw + other.yaw,
        }
    }
}

#[derive(Default)]
pub struct ThrottleInput(pub u16, pub u16);

impl core::fmt::Display for ThrottleInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.1)
    }
}

impl core::ops::Add for ThrottleInput {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let t0 = ((self.0 as u32 + other.0 as u32) / 2) as u16;
        let t1 = ((self.1 as u32 + other.1 as u32) / 2) as u16;
        Self(t0, t1)
    }
}

pub trait Controller {
    fn get_throttle(&self) -> ThrottleInput;
    fn get_input(&self) -> ControlSurfaceInput;
}

pub struct NoController;

impl Controller for NoController {
    fn get_throttle(&self) -> ThrottleInput {
        ThrottleInput::default()
    }

    fn get_input(&self) -> ControlSurfaceInput {
        ControlSurfaceInput::default()
    }
}
