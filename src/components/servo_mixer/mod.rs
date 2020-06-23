pub mod aircraft;
pub mod fixed_wing;

pub use aircraft::Aircraft;
pub use fixed_wing::FixedWing;

use crate::hal::controller::Controller;

pub struct AircraftController<A, RC, S, FC> {
    aircraft: A,
    remote_controller: RC,
    stablizer: S,
    stablizer_scale: u8,
    flight_controller: FC,
    flight_controller_scale: u8,
}

impl<A: Aircraft, RC: Controller, S: Controller, FC: Controller> AircraftController<A, RC, S, FC> {
    pub fn new(aircraft: A, remote_controller: RC, stablizer: S, flight_controller: FC) -> Self {
        Self {
            aircraft,
            remote_controller,
            stablizer,
            flight_controller,
            stablizer_scale: 30,
            flight_controller_scale: 30,
        }
    }

    pub fn set_stablizer_scale(&mut self, scale: u8) {
        self.stablizer_scale = scale;
    }

    pub fn set_flight_controller_scale(&mut self, scale: u8) {
        self.flight_controller_scale = scale;
    }

    pub fn run_once(&mut self) {
        let rc_input = self.remote_controller.get_input();
        let stablizer_input = self.stablizer.get_input().scale_down(self.stablizer_scale);
        let fc_input = self.flight_controller.get_input().scale_down(self.flight_controller_scale);
        let input = rc_input + stablizer_input + fc_input;

        let rc_throttle = self.remote_controller.get_throttle();
        let fc_throttle = self.flight_controller.get_throttle();
        let throttle = rc_throttle + fc_throttle;

        self.aircraft.control(throttle, input);
    }
}
