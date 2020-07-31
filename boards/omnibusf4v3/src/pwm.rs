use core::cmp::min;

use rs_flight::config::Outputs;
use rs_flight::drivers::pwm::PwmByIdentifier;
use stm32f4xx_hal::gpio::gpioa::{PA1, PA2, PA3, PA8};
use stm32f4xx_hal::gpio::gpiob::{PB0, PB1};
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::pwm;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::{prelude::*, stm32};

type Default = Input<Floating>;
type PWMs = (stm32::TIM1, stm32::TIM2, stm32::TIM3, stm32::TIM5);
type PINs = (PB0<Default>, PB1<Default>, PA2<Default>, PA3<Default>, PA1<Default>, PA8<Default>);

pub fn init(pwms: PWMs, pins: PINs, clocks: Clocks, cfg: &Outputs) -> impl PwmByIdentifier {
    let rate1 = cfg.get("PWM1").map(|o| o.rate()).unwrap_or(50) as u32;
    let rate2 = cfg.get("PWM2").map(|o| o.rate()).unwrap_or(50) as u32;

    let (tim1, tim2, tim3, tim5) = pwms;
    let (pb0, pb1, pa2, pa3, pa1, pa8) = pins;
    let pb0_1 = (pb0.into_alternate_af2(), pb1.into_alternate_af2());
    let rate = if rate1 > 0 && rate2 > 0 { min(rate1, rate2) } else { rate1 + rate2 };
    let (pwm1, pwm2) = pwm::tim3(tim3, pb0_1, clocks, rate.hz());

    let rate3 = cfg.get("PWM3").map(|o| o.rate()).unwrap_or(50) as u32;
    let rate4 = cfg.get("PWM4").map(|o| o.rate()).unwrap_or(50) as u32;
    let pa2_3 = (pa2.into_alternate_af1(), pa3.into_alternate_af1());
    let rate = if rate3 > 0 && rate4 > 0 { min(rate3, rate4) } else { rate3 + rate4 };
    let (pwm4, pwm3) = pwm::tim2(tim2, pa2_3, clocks, rate.hz());

    let rate = cfg.get("PWM5").map(|o| o.rate()).unwrap_or(50) as u32;
    let pwm5 = pwm::tim5(tim5, pa1.into_alternate_af2(), clocks, rate.hz());
    let rate = cfg.get("PWM6").map(|o| o.rate()).unwrap_or(50) as u32;
    let pwm6 = pwm::tim1(tim1, pa8.into_alternate_af1(), clocks, rate.hz());

    let mut pwms = (pwm1, pwm2, pwm3, pwm4, pwm5, pwm6);
    pwms.foreach(|pwm| pwm.enable());
    pwms
}
