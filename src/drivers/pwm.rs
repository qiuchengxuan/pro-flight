use core::cell::UnsafeCell;

use embedded_hal::PwmPin;

use crate::config::output::Identifier;

pub trait PWM = PwmPin<Duty = u16>;

pub trait PwmByIdentifier {
    fn get<'a>(&self, identifier: Identifier) -> Option<&mut dyn PWM>;
}

pub struct PWM6<P1, P2, P3, P4, P5, P6>(UnsafeCell<(P1, P2, P3, P4, P5, P6)>);

impl<P1, P2, P3, P4, P5, P6> PWM6<P1, P2, P3, P4, P5, P6> {
    pub fn new(pwms: (P1, P2, P3, P4, P5, P6)) -> Self {
        Self(UnsafeCell::new(pwms))
    }
}

impl<P1: PWM, P2: PWM, P3: PWM, P4: PWM, P5: PWM, P6: PWM> PwmByIdentifier
    for PWM6<P1, P2, P3, P4, P5, P6>
{
    fn get<'a>(&self, identifier: Identifier) -> Option<&mut dyn PWM> {
        match identifier {
            Identifier::PWM(index) => unsafe {
                match index {
                    1 => Some(&mut (&mut *self.0.get()).0),
                    2 => Some(&mut (&mut *self.0.get()).1),
                    3 => Some(&mut (&mut *self.0.get()).2),
                    4 => Some(&mut (&mut *self.0.get()).3),
                    5 => Some(&mut (&mut *self.0.get()).4),
                    6 => Some(&mut (&mut *self.0.get()).5),
                    _ => None,
                }
            },
        }
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
