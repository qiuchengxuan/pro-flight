use crate::datastructures::input::{Flaps, LandingGear, Pitch, Roll, Throttle, Yaw};

pub trait BasicInput {
    fn get_throttle(&self) -> Throttle;
    fn get_roll(&self) -> Roll;
    fn get_pitch(&self) -> Pitch;
    fn get_yaw(&self) -> Yaw;
}

pub trait FixedWingInput {
    fn get_flaps(&self) -> Flaps;
    fn get_landing_gear(&self) -> LandingGear;
}

pub struct NoInput;

impl BasicInput for NoInput {
    fn get_throttle(&self) -> Throttle {
        0
    }

    fn get_roll(&self) -> Roll {
        0
    }

    fn get_pitch(&self) -> Pitch {
        0
    }

    fn get_yaw(&self) -> Yaw {
        0
    }
}

impl FixedWingInput for NoInput {
    fn get_flaps(&self) -> Flaps {
        Flaps::Auto
    }

    fn get_landing_gear(&self) -> LandingGear {
        LandingGear::Up
    }
}
