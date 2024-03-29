use core::cmp::min;

use alloc::{boxed::Box, vec, vec::Vec};

use embedded_hal::PwmPin;
use pro_flight::config::peripherals::pwm::PWMs as Config;
use stm32f4xx_hal::{
    gpio::{
        gpioa::{PA1, PA2, PA3, PA8},
        gpiob::{PB0, PB1},
        Input,
    },
    pac,
    prelude::*,
    rcc::Clocks,
};

type PWMs = (pac::TIM1, pac::TIM2, pac::TIM3, pac::TIM5);
type PINs = (PB0<Input>, PB1<Input>, PA2<Input>, PA3<Input>, PA1<Input>, PA8<Input>);
type PwmPins = Vec<(&'static str, Box<dyn PwmPin<Duty = u16> + Send + 'static>)>;

pub fn init(pwms: PWMs, pins: PINs, clocks: &Clocks, cfg: &Config) -> PwmPins {
    let rate1 = cfg.get("PWM1").map(|o| o.rate()).unwrap_or(50) as u32;
    let rate2 = cfg.get("PWM2").map(|o| o.rate()).unwrap_or(50) as u32;

    let (tim1, tim2, tim3, tim5) = pwms;
    let (pb0, pb1, pa2, pa3, pa1, pa8) = pins;
    let pb0_1 = (pb0.into_alternate(), pb1.into_alternate());
    let rate = if rate1 > 0 && rate2 > 0 { min(rate1, rate2) } else { rate1 + rate2 };
    let (pwm1, pwm2) = tim3.pwm_hz(pb0_1, rate.Hz(), clocks).split();

    let rate3 = cfg.get("PWM3").map(|o| o.rate()).unwrap_or(50) as u32;
    let rate4 = cfg.get("PWM4").map(|o| o.rate()).unwrap_or(50) as u32;
    let pa2_3 = (pa2.into_alternate(), pa3.into_alternate());
    let rate = if rate3 > 0 && rate4 > 0 { min(rate3, rate4) } else { rate3 + rate4 };
    let (pwm4, pwm3) = tim2.pwm_hz(pa2_3, rate.Hz(), clocks).split();

    let rate = cfg.get("PWM5").map(|o| o.rate()).unwrap_or(50) as u32;
    let pwm5 = tim5.pwm_hz(pa1.into_alternate(), rate.Hz(), clocks).split();
    let rate = cfg.get("PWM6").map(|o| o.rate()).unwrap_or(50) as u32;
    let pwm6 = tim1.pwm_hz(pa8.into_alternate(), rate.Hz(), clocks).split();

    let mut pwms: PwmPins = vec![
        ("PWM1", Box::new(pwm1)),
        ("PWM2", Box::new(pwm2)),
        ("PWM3", Box::new(pwm3)),
        ("PWM4", Box::new(pwm4)),
        ("PWM5", Box::new(pwm5)),
        ("PWM6", Box::new(pwm6)),
    ];

    for (_, pwm) in pwms.iter_mut() {
        pwm.enable();
    }
    pwms
}
