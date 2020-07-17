use core::cell::UnsafeCell;

use embedded_hal::PwmPin;

use crate::config::output::Identifier;

pub trait PWM = PwmPin<Duty = u16>;

pub trait PwmByIdentifier {
    fn get<'a>(&self, identifier: Identifier) -> Option<&mut dyn PWM>;
}

macro_rules! pwms_impls {
    ($(
        $PWM:ident {
            $(($idx:tt) -> $P:ident)+
        }
    )+) => {
        $(
            pub struct $PWM<$($P,)+>(UnsafeCell<($($P,)+)>);

            impl<$($P,)+> $PWM<$($P,)+> {
                pub fn new(pwms: ($($P,)+)) -> Self {
                    Self(UnsafeCell::new(pwms))
                }
            }

            impl<$($P: PWM,)+> PwmByIdentifier for $PWM<$($P,)+> {
                fn get<'a>(&self, identifier: Identifier) -> Option<&mut dyn PWM> {
                    match identifier {
                        Identifier::PWM(index) => unsafe {
                            match index {
                                $(
                                    $idx => Some(&mut (&mut *self.0.get()).$idx),
                                )+
                                _ => None,
                            }
                        }
                    }
                }
            }
        )+
    }
}

pwms_impls! {
    PWM6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
}

pub struct DummyPWM;

static mut DUMMY_PWM: DummyPWM = DummyPWM {};

pub fn dummy_pwm() -> &'static mut DummyPWM {
    unsafe { &mut DUMMY_PWM }
}

impl PwmPin for DummyPWM {
    type Duty = u16;

    fn disable(&mut self) {}

    fn enable(&mut self) {}

    fn get_duty(&self) -> u16 {
        0
    }

    fn get_max_duty(&self) -> u16 {
        0
    }

    fn set_duty(&mut self, _: u16) {}
}
