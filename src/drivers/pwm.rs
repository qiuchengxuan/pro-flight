use embedded_hal::PwmPin;

use crate::config::output::Identifier;

pub trait PwmByIdentifier {
    fn with<F: FnMut(&mut dyn PwmPin<Duty = u16>)>(&mut self, identifier: Identifier, f: F);
    fn for_each<F: FnMut(&mut dyn PwmPin<Duty = u16>)>(&mut self, f: F);
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
            fn with<F: FnMut(&mut dyn PwmPin<Duty = u16>)>(&mut self, identifier: Identifier, mut f: F) {
                match identifier {
                    Identifier::PWM(id) => match (id - 1) {
                        $(
                            $idx => f(&mut self.$idx),
                        )+
                        _ => (),
                    }
                }
            }

            fn for_each<F: FnMut(&mut dyn PwmPin<Duty = u16>)>(&mut self, mut f: F) {
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
