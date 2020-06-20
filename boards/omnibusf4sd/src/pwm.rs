use core::mem::MaybeUninit;

use embedded_hal::PwmPin;
use rs_flight::components::servo_mixer::ServoMixer;
use stm32f4xx_hal::gpio::gpioa::{PA1, PA2, PA3, PA8};
use stm32f4xx_hal::gpio::gpiob::{PB0, PB1};
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::pwm;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::{prelude::*, stm32};

type Def = Input<Floating>;
type PWMs = (stm32::TIM1, stm32::TIM2, stm32::TIM3, stm32::TIM5);
type PINs = (PB0<Def>, PB1<Def>, PA2<Def>, PA3<Def>, PA1<Def>, PA8<Def>);

pub fn init(pwms: PWMs, pins: PINs, clocks: Clocks) -> ServoMixer<'static> {
    let (tim1, tim2, tim3, tim5) = pwms;
    let (pb0, pb1, pa2, pa3, pa1, pa8) = pins;
    let pb0_1 = (pb0.into_alternate_af2(), pb1.into_alternate_af2());
    let (pwm1, pwm2) = pwm::tim3(tim3, pb0_1, clocks, 50.hz());

    let pa2_3 = (pa2.into_alternate_af1(), pa3.into_alternate_af1());
    let (pwm3, pwm4) = pwm::tim2(tim2, pa2_3, clocks, 50.hz());
    let pwm5 = pwm::tim5(tim5, pa1.into_alternate_af2(), clocks, 50.hz());
    let pwm6 = pwm::tim1(tim1, pa8.into_alternate_af1(), clocks, 50.hz());

    static mut PWM1: MaybeUninit<pwm::PwmChannels<stm32::TIM3, pwm::C3>> = MaybeUninit::uninit();
    static mut PWM2: MaybeUninit<pwm::PwmChannels<stm32::TIM3, pwm::C4>> = MaybeUninit::uninit();
    static mut PWM3: MaybeUninit<pwm::PwmChannels<stm32::TIM2, pwm::C3>> = MaybeUninit::uninit();
    static mut PWM4: MaybeUninit<pwm::PwmChannels<stm32::TIM2, pwm::C4>> = MaybeUninit::uninit();
    static mut PWM5: MaybeUninit<pwm::PwmChannels<stm32::TIM5, pwm::C2>> = MaybeUninit::uninit();
    static mut PWM6: MaybeUninit<pwm::PwmChannels<stm32::TIM1, pwm::C1>> = MaybeUninit::uninit();
    static mut PWMS: MaybeUninit<[&mut dyn PwmPin<Duty = u16>; 6]> = MaybeUninit::uninit();
    unsafe {
        PWM1 = MaybeUninit::new(pwm1);
        PWM2 = MaybeUninit::new(pwm2);
        PWM3 = MaybeUninit::new(pwm3);
        PWM4 = MaybeUninit::new(pwm4);
        PWM5 = MaybeUninit::new(pwm5);
        PWM6 = MaybeUninit::new(pwm6);
        PWMS = MaybeUninit::new([
            &mut *PWM1.as_mut_ptr(),
            &mut *PWM2.as_mut_ptr(),
            &mut *PWM3.as_mut_ptr(),
            &mut *PWM4.as_mut_ptr(),
            &mut *PWM5.as_mut_ptr(),
            &mut *PWM6.as_mut_ptr(),
        ]);
        ServoMixer::new(&mut *PWMS.as_mut_ptr())
    }
}
