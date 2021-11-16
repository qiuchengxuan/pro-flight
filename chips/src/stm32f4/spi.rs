#[derive(Copy, Clone)]
pub struct BaudrateControl(pub u32);

impl BaudrateControl {
    pub fn new(pclk: u32, freq: u32) -> Self {
        Self(match pclk / freq {
            0 => unreachable!(),
            1..=2 => 0b000,
            3..=4 => 0b001,
            5..=8 => 0b010,
            9..=16 => 0b011,
            17..=32 => 0b100,
            33..=64 => 0b101,
            65..=128 => 0b110,
            _ => 0b111,
        })
    }
}

#[macro_export]
macro_rules! __define_spi {
    (
        $spi:ident =>
        ($gpio:ident, $sclk:ident, $miso:ident, $mosi:ident, $af:ident, $into_af:ident)
    ) => {
        type $sclk = gpio::$gpio::$sclk<Input<Floating>>;
        type $miso = gpio::$gpio::$miso<Input<Floating>>;
        type $mosi = gpio::$gpio::$mosi<Input<Floating>>;

        pub struct $spi<SPI: SpiMap> {
            sr: SPI::CSpiSr,
            dr: SPI::CSpiDr,
            cr2: SPI::SSpiCr2,
            sclk: gpio::$gpio::$sclk<Alternate<gpio::$af>>,
            miso: gpio::$gpio::$miso<Alternate<gpio::$af>>,
            mosi: gpio::$gpio::$mosi<Alternate<gpio::$af>>,
        }

        impl $spi<spi::$spi> {
            pub fn new(
                regs: SpiPeriph<spi::$spi>,
                pins: ($sclk, $miso, $mosi),
                baudrate: $crate::stm32f4::spi::BaudrateControl,
                mode: Mode,
            ) -> Self {
                let (sclk, miso, mosi) = pins;
                regs.rcc_busenr_spien.set_bit();
                regs.spi_cr1.modify(|r| {
                    if mode.polarity == Polarity::IdleHigh {
                        r.set_cpol();
                    }
                    if mode.phase == Phase::CaptureOnSecondTransition {
                        r.set_cpha();
                    }
                    r.write_br(baudrate.0).set_ssm().set_ssi().set_mstr().set_spe()
                });
                regs.spi_cr2.store(|r| r.set_rxneie().set_errie());
                let (sclk, miso, mosi) = (sclk.$into_af(), miso.$into_af(), mosi.$into_af());
                let (sr, dr) = (regs.spi_sr.into_copy(), regs.spi_dr.into_copy());
                Self { sr, dr, cr2: regs.spi_cr2, sclk, miso, mosi }
            }

            fn status(&mut self) -> Result<u32, Error> {
                loop {
                    let status = self.sr.load();
                    return match () {
                        _ if status.ovr() => Err(Error::Overrun),
                        _ if status.modf() => Err(Error::ModeFault),
                        _ if status.crcerr() => Err(Error::Crc),
                        _ if status.rxne() => Ok(self.dr.load().dr()),
                        _ => continue,
                    };
                }
            }
        }

        impl embedded_hal::blocking::spi::Transfer<u8> for $spi<spi::$spi> {
            type Error = Error;

            fn transfer<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a [u8], Error> {
                for i in 0..bytes.len() {
                    self.dr.store(|r| r.write_dr(bytes[i] as u32));
                    bytes[i] = self.status()? as u8;
                }
                Ok(bytes)
            }
        }

        impl embedded_hal::blocking::spi::Write<u8> for $spi<spi::$spi> {
            type Error = Error;

            fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
                for i in 0..bytes.len() {
                    self.dr.store(|r| r.write_dr(bytes[i] as u32));
                    self.status()?;
                }
                Ok(())
            }
        }

        impl dma::Peripheral for $spi<spi::$spi> {
            fn enable_dma(&mut self) {
                self.cr2.modify(|r| r.set_txdmaen().set_rxdmaen());
            }

            fn address(&mut self) -> usize {
                self.dr.as_mut_ptr() as usize
            }

            fn word_size(&self) -> usize {
                core::mem::size_of::<u8>()
            }
        }
    };
}

#[macro_export]
macro_rules! __define_spis {
    () => {};
    (
        $spi:ident => (
            $gpio:ident, $sclk:ident, $miso:ident, $mosi:ident, $af:ident, $into_af:ident
        )
        $($spis:ident => (
            $gpios:ident, $sclks:ident, $misos:ident, $mosis:ident, $afs:ident, $into_afs:ident
        ))*
    ) => {
        __define_spi!{$spi => ($gpio, $sclk, $miso, $mosi, $af, $into_af)}
        __define_spis!{$($spis => ($gpios, $sclks, $misos, $mosis, $afs, $into_afs))*}
    }
}

#[macro_export]
macro_rules! define_spis {
    ($($spi:ident => (
        $gpio:ident, $sclk:ident, $miso:ident, $mosi:ident, $af:ident, $into_af:ident
    ))+) => {
        use drone_core::sync::spsc::oneshot;
        use drone_core::fib::{FiberFuture, FiberFn, FiberState};
        use drone_cortexm::{fib, reg::prelude::*, thr::prelude::*, thr::ThrNvic};
        use drone_stm32_map::periph::spi::{self, traits::*, SpiPeriph, SpiMap};
        use embedded_hal::spi::{Mode, Phase, Polarity};
        use stm32f4xx_hal::gpio::{self, Alternate, Floating, Input, Output, PullUp, PushPull};

        use hal::dma;

        #[derive(Copy, Clone, Debug)]
        pub enum Error {
            Overrun,
            ModeFault,
            Crc,
        }

        type R = Result<u32, Error>;

        __define_spis!{$($spi => ($gpio, $sclk, $miso, $mosi, $af, $into_af))+}
    };
}
