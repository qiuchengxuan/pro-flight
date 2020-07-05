use core::mem::MaybeUninit;

use bmp280::bus::Bus;
use bmp280::measurement::{Calibration, RawPressure, RawTemperature};
use bmp280::registers::{PressureOversampling, TemperatureOversampling};
use bmp280::{Mode, BMP280};
use embedded_hal::blocking::delay::DelayMs;

use crate::alloc;
use crate::datastructures::ring_buffer::{RingBuffer, RingBufferReader};
use crate::hal::sensors::Pressure;

static mut CALIBRATION: MaybeUninit<Calibration> = MaybeUninit::uninit();
static mut RING_BUFFER: MaybeUninit<RingBuffer<Pressure>> = MaybeUninit::uninit();

pub unsafe fn on_dma_receive(dma_buffer: &[u8; 8]) {
    let calibration = &*CALIBRATION.as_ptr();
    let raw_pressure = RawPressure::from_bytes(&dma_buffer[2..]);
    let t_fine = RawTemperature::from_bytes(&dma_buffer[5..]).t_fine(calibration);
    let pressure = raw_pressure.compensated(t_fine, calibration);
    let ring_buffer = &mut *RING_BUFFER.as_mut_ptr();
    ring_buffer.write(Pressure(pressure));
}

pub fn init_ring() -> RingBufferReader<'static, Pressure> {
    let buffer = alloc::into_static([Pressure(0); 8], alloc::AllocateType::Generic).unwrap();
    unsafe { RING_BUFFER = MaybeUninit::new(RingBuffer::new(buffer)) };
    RingBufferReader::new(unsafe { &*RING_BUFFER.as_ptr() })
}

pub fn init<E, BUS, D>(bus: BUS, delay: &mut D) -> Result<bool, E>
where
    BUS: Bus<Error = E>,
    D: DelayMs<u8>,
{
    let mut bmp280 = BMP280::new(bus);
    bmp280.reset(delay)?;
    if !bmp280.verify()? {
        return Ok(false);
    }
    info!("BMP280 detected");
    bmp280.set_pressure_oversampling(PressureOversampling::StandardResolution)?;
    bmp280.set_temperature_oversampling(TemperatureOversampling::UltraLowPower)?;
    bmp280.set_mode(Mode::Normal)?;
    bmp280.set_standby_time(50)?; // at 20hz
    bmp280.set_iir_filter(8)?;
    let calibration = bmp280.read_calibration()?;
    unsafe { CALIBRATION = MaybeUninit::new(calibration) };
    Ok(true)
}
