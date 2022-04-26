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
    toggles: config::inputs::Toggles,
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

fn unsigned(value: i16) -> u16 {
    (value as i32 - i16::MIN as i32) as u16
}

impl ControlMatrix {
    fn reset(&mut self) {
        let inputs = &config::get().inputs;
        self.axes = inputs.axes.clone();
        self.toggles = inputs.toggles.clone();
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
                AxisType::Throttle => control.axes.throttle = unsigned(ch),
                AxisType::Roll => control.axes.roll = ch,
                AxisType::Pitch => control.axes.pitch = ch,
                AxisType::Yaw => control.axes.yaw = ch,
            }
        }
        for toggle in self.toggles.0.iter() {
            if toggle.channel as usize > channels.len() {
                continue;
            }
            let ch = unsigned(channels[toggle.channel as usize]);
            let index = ch / (u16::MAX / toggle.choices.len() as u16);
            let command = toggle.choices[index as usize];
            control.commands.push(command).ok();
        }
        datastore::acquire().write_control(control);
    }
}
