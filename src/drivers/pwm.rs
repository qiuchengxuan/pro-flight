use embedded_hal::PwmPin;

pub trait PWM = PwmPin<Duty = u16>;

pub trait PwmByName {
    fn get<'a>(&mut self, name: &'a [u8]) -> Option<&mut dyn PWM>;
}

pub struct PWM6<P1, P2, P3, P4, P5, P6>(pub P1, pub P2, pub P3, pub P4, pub P5, pub P6);

impl<P1: PWM, P2: PWM, P3: PWM, P4: PWM, P5: PWM, P6: PWM> PwmByName
    for PWM6<P1, P2, P3, P4, P5, P6>
{
    fn get<'a>(&mut self, name: &'a [u8]) -> Option<&mut dyn PWM> {
        match name {
            b"PWM1" => Some(&mut self.0),
            b"PWM2" => Some(&mut self.1),
            b"PWM3" => Some(&mut self.3),
            b"PWM4" => Some(&mut self.3),
            b"PWM5" => Some(&mut self.4),
            b"PWM6" => Some(&mut self.5),
            _ => None,
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
