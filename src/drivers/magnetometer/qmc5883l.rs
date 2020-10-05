use alloc::rc::Rc;

use embedded_hal::blocking::i2c::{Write, WriteRead};
use qmc5883l::{FieldRange, OutputDataRate, OversampleRate, QMC5883L};

use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::data_source::singular::SingularData;
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::{Axes, Magnetism};
use crate::drivers::magnetometer::global;

pub struct QMC5883LWrapper<I2C>(QMC5883L<I2C>);

impl<E, I2C: Write<Error = E> + WriteRead<Error = E>> Schedulable for QMC5883LWrapper<I2C> {
    fn rate(&self) -> Rate {
        200
    }

    fn schedule(&mut self) -> bool {
        if let Some(magnetometer) = unsafe { &mut global::MAGNETOMETER } {
            if let Some((x, y, z)) = self.0.read_magnetism().ok() {
                let magnetism = Magnetism {
                    axes: Axes { x: x as i32, y: y as i32, z: z as i32 },
                    sensitive: FieldRange::Range8Gauss.sensitive() as i32,
                };
                magnetometer.write(magnetism);
            }
        }
        true
    }
}

pub fn init<E, I2C>(mut i2c: I2C) -> Result<Option<QMC5883LWrapper<I2C>>, E>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    if !QMC5883L::probe(&mut i2c)? {
        return Ok(None);
    }
    let mut qmc5883l = QMC5883L::new(i2c);
    info!("QMC5883L detected");
    qmc5883l.reset()?;
    qmc5883l.set_field_range(FieldRange::Range8Gauss)?;
    qmc5883l.set_output_data_rate(OutputDataRate::Rate200Hz)?;
    qmc5883l.set_oversample(OversampleRate::Rate512)?;
    qmc5883l.continuous()?;
    unsafe { global::MAGNETOMETER = Some(Rc::new(SingularData::default())) }
    Ok(Some(QMC5883LWrapper(qmc5883l)))
}
