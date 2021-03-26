use core::fmt::Debug;

use chips::hal::dma::{Buffer, BufferDescriptor, Peripheral, DMA};
use chips::stm32f4::delay::TickDelay;
use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::thr::ThrNvic;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
pub use mpu6000::SPI_MODE;
use mpu6000::{
    bus::{SpiBus, SpiError},
    registers::Register,
    MPU6000,
};
use pro_flight::{
    config,
    datastructures::measurement::{Acceleration, Measurement},
    drivers::mpu6000::{Convertor, MPU6000Init, NUM_MEASUREMENT_REGS},
    sys::timer::SysTimer,
};
use stm32f4xx_hal::gpio::ExtiPin;

const SAMPLE_RATE: usize = 1000;

pub struct DmaSpiMPU6000<SPI, CS, INT, RX, TX, THR> {
    pub spi: SPI,
    pub cs: CS,
    pub int: INT,
    pub rx: RX,
    pub tx: TX,
    pub thread: THR,
}

impl<E: Debug, PE: Debug, SPI, CS, INT, RX, TX, THR> DmaSpiMPU6000<SPI, CS, INT, RX, TX, THR>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + Peripheral + Send + 'static,
    CS: OutputPin<Error = PE> + Send + 'static + Unpin,
    INT: ExtiPin + Send + 'static,
    RX: DMA,
    TX: DMA,
    THR: ThrNvic,
{
    pub fn init(self, mut handler: impl FnMut(Acceleration, Measurement) + 'static + Send) {
        let mut mpu6000 = MPU6000::new(SpiBus::new(self.spi, self.cs, TickDelay {}));
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

        let (mut spi, mut cs, _) = mpu6000.free().free();

        let mut buffer = Buffer::<u8, { 1 + NUM_MEASUREMENT_REGS }>::default();
        let mut cs_ = unsafe { core::ptr::read(&cs as *const _ as *const CS) };
        let convertor = Convertor::default();
        let rotation = config::get().board.rotation;
        buffer.set_transfer_done(Box::leak(Box::new(move |bytes| {
            cs_.set_high().ok();
            let (acceleration, gyro) = convertor.convert(&bytes[1..], rotation);
            handler(acceleration, gyro);
        })));
        let mut rx_bd: BufferDescriptor<u8> = Box::leak(Box::new(buffer)).into();
        let buffer = rx_bd.borrow_mut().unwrap();
        info!("MPU6000 detected, Init DMA address at {:x}", buffer.as_ptr() as usize);

        let buffer = Buffer::<u8, 1>::new([Register::AccelerometerXHigh as u8 | 0x80]);
        let tx_bd: BufferDescriptor<u8> = Box::leak(Box::new(buffer)).into();

        let mut rx = self.rx;
        rx.setup_peripheral(3, &mut spi);
        let mut tx = self.tx;
        tx.setup_peripheral(3, &mut spi);
        let mut int = self.int;

        self.thread.add_fib(new_fn(move || {
            int.clear_interrupt_pending_bit();
            cs.set_low().ok();
            rx.rx(&rx_bd, false);
            tx.tx(&tx_bd, Some(1 + NUM_MEASUREMENT_REGS));
            Yielded::<(), ()>(())
        }));
        self.thread.enable_int()
    }
}
