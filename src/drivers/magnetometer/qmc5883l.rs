use embedded_hal::blocking::i2c::{Write, WriteRead};

use qmc5883l::{FieldRange, OutputDataRate, OversampleRate, QMC5883L};

use crate::datastructures::data_source::singular::SingularData;
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::{Axes, Magnetism};
use crate::drivers::magnetometer::global;

pub fn dma_read_bytes() -> &'static [u8] {
    static READ_BYTES: [u8; 2] = [0xD << 1 | 1, 0x6];
    &READ_BYTES
}

pub fn on_dma_receive(bytes: &[u8]) {
    if bytes[1] & 0x1 == 0 {
        return; // Data not Ready
    }
    if bytes[1] & 0x2 > 0 {
        return; // Overflow
    }
    let axes: &[i16] = unsafe { core::mem::transmute(&bytes[2..]) };
    if let Some(magnetometer) = unsafe { &mut global::MAGNETOMETER } {
        let magnetism = Magnetism {
            axes: Axes {
                x: i16::from_le(axes[0]) as i32,
                y: i16::from_le(axes[1]) as i32,
                z: i16::from_le(axes[2]) as i32,
            },
            sensitive: (i16::MAX / 8) as i32,
        };
        magnetometer.write(magnetism);
    }
}

pub fn init<E>(i2c: (impl Write<Error = E> + WriteRead<Error = E>)) -> Result<bool, E> {
    let mut qmc5883l = match QMC5883L::new(i2c) {
        Ok(qmc5883l) => qmc5883l,
        Err(_) => return Ok(false),
    };
    info!("Found QMC5883L");
    qmc5883l.set_field_range(FieldRange::Range8Gauss)?;
    qmc5883l.set_output_data_rate(OutputDataRate::Rate200Hz)?;
    qmc5883l.set_oversample(OversampleRate::Rate512)?;
    qmc5883l.continuous()?;
    unsafe { global::MAGNETOMETER = Some(SingularData::default()) }
    Ok(true)
}
