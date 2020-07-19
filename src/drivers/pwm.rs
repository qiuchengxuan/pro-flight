use embedded_hal::PwmPin;

use crate::config::output::Identifier;

pub trait PwmByIdentifier {
    fn with<F: FnMut(&mut dyn PwmPin<Duty = u16>)>(&mut self, identifier: Identifier, f: F);
}

macro_rules! pwms_impls {
    ($(
        $PWM:ident {
            $(($idx:tt) -> $P:ident)+
        }
    )+) => {
        $(
            pub struct $PWM<$($P,)+>($($P,)+);

            impl<$($P,)+> $PWM<$($P,)+> {
                pub fn new(pwms: ($($P,)+)) -> Self {
                    Self($(pwms.$idx,)+)
                }
            }

            impl<$($P: PwmPin<Duty = u16>,)+> PwmByIdentifier for $PWM<$($P,)+> {
                fn with<F: FnMut(&mut dyn PwmPin<Duty = u16>)>(&mut self, identifier: Identifier, mut f: F){
                    match identifier {
                        Identifier::PWM(index) => match index {
                            $(
                                $idx => f(&mut self.$idx),
                            )+
                            _ => (),
                        }
                    }
                }
            }
        )+
    }
}

pwms_impls! {
    PWM6 {
        (0) -> P0
        (1) -> P1
        (2) -> P2
        (3) -> P3
        (4) -> P4
        (5) -> P5
    }
}
