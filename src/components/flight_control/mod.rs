pub mod aircraft;
pub mod airplane;

pub use aircraft::Aircraft;
pub use airplane::Airplane;

use crate::hal::controller::Controller;

pub struct FlightControl<A, RC, S, AP> {
    aircraft: A,
    receiver: RC,
    stablizer: S,
    stablizer_scale: u8,
    autopilot: AP,
    autopilot_scale: u8,
}

impl<A: Aircraft, RC: Controller, S: Controller, AP: Controller> FlightController<A, RC, S, FC> {
    pub fn new(aircraft: A, receiver: RC, stablizer: S, autopilot: AP) -> Self {
        Self { aircraft, receiver, stablizer, autopilot, stablizer_scale: 30, autopilot_scale: 30 }
    }

    pub fn set_stablizer_scale(&mut self, scale: u8) {
        self.stablizer_scale = scale;
    }

    pub fn set_flight_controller_scale(&mut self, scale: u8) {
        self.autopilot_scale = scale;
    }

    pub fn run_once(&mut self) {
        let receiver_input = self.receiver.get_input();
        let stablizer_input = self.stablizer.get_input().scale_down(self.stablizer_scale);
        let fc_input = self.autopilot.get_input().scale_down(self.autopilot_scale);
        let input = receiver_input + stablizer_input + fc_input;

        let rc_throttle = self.receiver.get_throttle();
        let fc_throttle = self.autopilot.get_throttle();
        let throttle = rc_throttle + fc_throttle;

        self.aircraft.control(throttle, input);
    }
}
