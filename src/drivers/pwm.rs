use embedded_hal::PwmPin;

use crate::config::output::Identifier;

pub trait PwmByIdentifier {
    fn with(&mut self, identifier: Identifier, f: impl FnMut(&mut dyn PwmPin<Duty = u16>));
    fn foreach(&mut self, f: impl FnMut(&mut dyn PwmPin<Duty = u16>));
}

macro_rules! peel {
    (($idx0:tt) -> $P0:ident
     $(
         ($idx:tt) -> $P:ident
     )*
    ) => (pwms! { $(($idx) -> $P)* })
}

macro_rules! pwms {
    () => ();
    ($(($idx:tt) -> $P:ident)+) => {
        impl<$($P: PwmPin<Duty = u16>,)+> PwmByIdentifier for ($($P,)+) {
            fn with(&mut self, identifier: Identifier, mut f: impl FnMut(&mut dyn PwmPin<Duty = u16>)) {
                match identifier {
                    $(
                        Identifier::PWM($idx) => f(&mut self.$idx),
                    )+
                    _ => (),
                }
            }

            fn foreach(&mut self, mut f: impl FnMut(&mut dyn PwmPin<Duty = u16>)) {
                $(f(&mut self.$idx);)+
            }
        }
        peel!{ $(($idx) -> $P)+ }
    }
}

pwms! {
   (5) -> P5
   (4) -> P4
   (3) -> P3
   (2) -> P2
   (1) -> P1
   (0) -> P0
}
