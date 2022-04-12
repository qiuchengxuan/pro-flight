use crate::{
    config, datastore,
    types::control::{AxisType, Control, RSSI},
};

pub const MAX_CHANNEL: usize = 18;

pub struct RawControl {
    pub rssi: RSSI,
    pub channels: [i16; MAX_CHANNEL],
}

#[derive(Default)]
pub struct ControlMatrix {
    config_iteration: usize,
    axes: config::inputs::Axes,
}

fn scale(value: i16, scale: u8) -> i16 {
    let scaled = value as i32 * scale as i32 / 100;
    if scaled > i16::MAX as i32 {
        i16::MAX
    } else if scaled < i16::MIN as i32 {
        i16::MIN
    } else {
        scaled as i16
    }
}

impl ControlMatrix {
    fn reset(&mut self) {
        self.axes = config::get().inputs.axes.clone();
    }

    pub fn read(&mut self, channels: &[i16; MAX_CHANNEL]) {
        if self.config_iteration != config::iteration() {
            self.reset();
        }
        let mut control = Control::default();
        for (axis_type, axis) in self.axes.0.iter() {
            if axis.channel as usize > channels.len() {
                continue;
            }
            let ch = scale(channels[axis.channel as usize], axis.scale.0);
            match axis_type {
                AxisType::Throttle => control.axes.throttle = (ch as i32 - i16::MIN as i32) as u16,
                AxisType::Roll => control.axes.roll = ch,
                AxisType::Pitch => control.axes.pitch = ch,
                AxisType::Yaw => control.axes.yaw = ch,
            }
        }
        datastore::acquire().write_control(control);
    }
}
