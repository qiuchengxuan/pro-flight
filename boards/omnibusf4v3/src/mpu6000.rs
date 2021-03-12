use core::fmt::Debug;

use chips::stm32f4::delay::TickDelay;
use chips::stm32f4::dma::{self, TRxDMA};
use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::thr::ThrNvic;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use mpu6000::bus::{SpiBus, SpiError};
use mpu6000::registers::Register;
use mpu6000::MPU6000;
pub use mpu6000::SPI_MODE;
use pro_flight::config;
use pro_flight::drivers::mpu6000::{Convertor, MPU6000Init, NUM_MEASUREMENT_REGS};
use pro_flight::sys::timer::SysTimer;
use stm32f4xx_hal::gpio::ExtiPin;

const SAMPLE_RATE: usize = 1000;

pub fn init<E, PE, SPI, CS, INT, RX, TX>(
    spi1: SPI,
    cs: CS,
    mut int: INT,
    mut dma: TRxDMA<TX, RX>,
    thread: impl ThrNvic,
) where
    E: Debug,
    PE: Debug,
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + dma::Peripheral + Send + 'static,
    CS: OutputPin<Error = PE> + Send + 'static + Unpin,
    INT: ExtiPin + Send + 'static,
    RX: dma::Receive + dma::Channel + Send + 'static,
    TX: dma::Transmit + dma::Channel + Send + 'static,
{
    let mut mpu6000 = MPU6000::new(SpiBus::new(spi1, cs, TickDelay {}));
    let mut delay = SysTimer::new();
    let result: Result<(), SpiError<E, E, PE>> = (|| {
        mpu6000.reset(&mut delay)?;
        if !mpu6000.verify()? {
            error!("MPU6000 not detected");
            return Ok(());
        }
        mpu6000.init(SAMPLE_RATE as u16)
    })();
    if let Some(error) = result.err() {
        error!("MPU6000 init failed: {:?}", error);
        return;
    }

    let (mut spi1, mut cs, _) = mpu6000.free().free();

    let rx_buffer = Box::leak(Box::new([0u8; 2 + NUM_MEASUREMENT_REGS]));
    info!("MPU6000 detected, Init DMA address at {:x}", rx_buffer.as_ptr() as usize);

    dma.rx.setup_memory(&mut rx_buffer[1..]);
    dma.rx.setup_peripheral(3, &mut spi1);

    static READ_REG: u8 = Register::AccelerometerXHigh as u8 | 0x80;
    dma.tx.setup_memory(core::slice::from_ref(&READ_REG), Some(1 + NUM_MEASUREMENT_REGS));
    dma.tx.setup_peripheral(3, &mut spi1);

    let mut cs_ = unsafe { core::ptr::read(&cs as *const _ as *const CS) };
    let convertor = Convertor::default();
    let rotation = config::get().board.rotation;
    dma.rx.on_finished(move |bytes| {
        cs_.set_high().ok();
        let (_acceleration, _gyro) = convertor.convert(&bytes[1..], rotation);
    });

    thread.add_fib(new_fn(move || {
        int.clear_interrupt_pending_bit();
        cs.set_low().ok();
        dma.start();
        Yielded::<(), ()>(())
    }));
    thread.enable_int()
}
