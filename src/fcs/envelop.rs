#[cfg(not(any(test, feature = "std")))]
use micromath::F32Ext;
use nalgebra::Vector3;

use crate::types::measurement::Attitude;

pub struct Envelop {
    max_roll: f32,
    min_pitch: f32,
    max_pitch: f32,
    prev_roll_command: f32,
    hold_bank_angle: f32,
}

type Control = Vector3<f32>;

impl Envelop {
    pub fn new() -> Self {
        let config = crate::config::get().fcs.envelop;
        let max_roll = config.max_roll as f32;
        let (min_pitch, max_pitch) = (config.min_pitch as f32, config.max_pitch as f32);
        Self { max_roll, min_pitch, max_pitch, prev_roll_command: 0.0, hold_bank_angle: 0.0 }
    }

    pub fn restrict(&mut self, input: Control, atti: Attitude, g: f32) -> Control {
        let mut output = input;

        if input.x.abs() < 0.1 && self.prev_roll_command.abs() > 0.1 {
            self.hold_bank_angle = atti.roll;
        }
        self.prev_roll_command = input.x;

        output.y = match () {
            _ if atti.pitch < self.min_pitch => -f32::max(atti.pitch - self.min_pitch, -10.0),
            _ if atti.pitch > self.max_pitch => -f32::min(atti.pitch - self.max_pitch, 10.0),
            _ if input.y.abs() < 0.1 => ((1.0 - g) * 100.0).clamp(-10.0, 10.0),
            _ => input.y,
        };

        if atti.roll.abs() > self.max_roll {
            output.x = -f32::min(atti.roll.abs() - self.max_roll, 25.0).copysign(atti.roll);
        } else if input.x.abs() < 0.1 {
            if atti.roll.abs() < 33.0 {
                output.x = (self.hold_bank_angle - atti.roll).clamp(-25.0, 25.0);
            } else if input.y.abs() < 0.1 {
                output.x = -f32::min(atti.roll.abs() - 33.0, 25.0).copysign(atti.roll);
            }
        }
        output
    }
}

mod test {
    #[test]
    fn test_envelop_roll() {
        use nalgebra::Vector3;

        use crate::types::measurement::Attitude;

        crate::config::reset();

        let mut envelop = super::Envelop::new();
        let input = Vector3::new(0.0, 0.0, 0.0);
        let attitude = Attitude::new(0.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(0.0, 0.0, 0.0));

        // Should reduce bank angle
        let attitude = Attitude::new(10.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(-10.0, 0.0, 0.0));

        // Should not reduce bank angle since rolling
        let input = Vector3::new(10.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(10.0, 0.0, 0.0));

        let input = Vector3::new(0.0, 0.0, 0.0);
        envelop.restrict(input, attitude, 1.0);

        // Should keep bank angle at 10°
        let attitude = Attitude::new(11.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(-1.0, 0.0, 0.0));

        // Should reduce bank angle to 33°
        let attitude = Attitude::new(35.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(-2.0, 0.0, 0.0));

        // Should not output roll with pitch input
        let attitude = Attitude::new(35.0, 0.0, 0.0);
        let input = Vector3::new(0.0, 10.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(0.0, 10.0, 0.0));

        // Exceeding maximum bank angle
        let input = Vector3::new(10.0, 0.0, 0.0);
        let attitude = Attitude::new(70.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(-3.0, 0.0, 0.0));

        // Should not exceed 25°/s
        let attitude = Attitude::new(100.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(-25.0, 0.0, 0.0));
        let attitude = Attitude::new(-100.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(25.0, 0.0, 0.0));
    }

    #[test]
    fn test_envelop_pitch() {
        use nalgebra::Vector3;

        use crate::types::measurement::Attitude;

        crate::config::reset();

        let mut envelop = super::Envelop::new();

        // Should reduce pitch angle
        let input = Vector3::new(0.0, 0.0, 0.0);
        let attitude = Attitude::new(0.0, 35.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(0.0, -5.0, 0.0));

        let input = Vector3::new(0.0, 10.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(0.0, -5.0, 0.0));

        // Should maintain constant G without input
        let input = Vector3::new(0.0, 0.0, 0.0);
        let attitude = Attitude::new(0.0, 0.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.1), Vector3::new(0.0, -10.0, 0.0));
        assert_eq!(envelop.restrict(input, attitude, 1.2), Vector3::new(0.0, -10.0, 0.0));
        assert_eq!(envelop.restrict(input, attitude, 0.9), Vector3::new(0.0, 10.0, 0.0));
        assert_eq!(envelop.restrict(input, attitude, -0.1), Vector3::new(0.0, 10.0, 0.0));

        // Override pitch command with high pitch attitude
        let input = Vector3::new(0.0, 10.0, 0.0);
        let attitude = Attitude::new(0.0, 40.0, 0.0);
        assert_eq!(envelop.restrict(input, attitude, 1.0), Vector3::new(0.0, -10.0, 0.0));
    }
}
