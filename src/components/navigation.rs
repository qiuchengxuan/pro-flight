use alloc::boxed::Box;
use alloc::rc::Rc;

use crate::algorithm::ComplementaryFilter;
use crate::alloc;
use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::coordinate::{Displacement, Position};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{AgingStaticData, DataWriter, StaticData};
use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::unit::Meter;
use crate::datastructures::measurement::{Altitude, VelocityVector};
use crate::datastructures::waypoint::{Steerpoint, Waypoint};

const CURRENT: usize = 0;
const HOME: usize = 0;
const MAX_WAYPOINT: usize = 32;

pub struct Navigation<A, S> {
    altimeter: A,
    speedometer: S,

    gnss: Option<Box<dyn AgingStaticData<Position>>>,

    velocity: VelocityVector<f32, Meter>,
    filters: [ComplementaryFilter<f32>; 3],
    displacement: (f32, f32, f32),
    waypoints: [Waypoint; MAX_WAYPOINT],
    displacements: [Displacement<Meter>; MAX_WAYPOINT],
    output: Rc<SingularData<(Position, Steerpoint)>>,
    next_waypoint: u8,
    max_waypoint: u8,
}

impl<A, S> Navigation<A, S> {
    pub fn new(altimeter: A, speedometer: S) -> Self {
        Self {
            altimeter,
            speedometer,
            gnss: None,
            velocity: VelocityVector::default(),
            filters: [ComplementaryFilter::new(0.1, 0.02); 3],
            displacement: (0.0, 0.0, 0.0),
            waypoints: [Waypoint::default(); MAX_WAYPOINT],
            displacements: [Displacement::default(); MAX_WAYPOINT],
            output: Rc::new(SingularData::default()),
            next_waypoint: HOME as u8,
            max_waypoint: 1,
        }
    }

    pub fn reader(&self) -> SingularDataSource<(Position, Steerpoint)> {
        SingularDataSource::new(&self.output)
    }

    pub fn set_gnss(&mut self, gnss: Box<dyn AgingStaticData<Position>>) {
        self.gnss = Some(gnss)
    }

    pub fn next_waypoint(&mut self) {
        self.next_waypoint = (self.next_waypoint + 1) % self.max_waypoint;
    }

    pub fn update_waypoint(&mut self, index: usize, waypoint: Waypoint) {
        if index >= MAX_WAYPOINT || index != self.max_waypoint as usize + 1 {
            return;
        }
        self.waypoints[index] = waypoint;
        self.max_waypoint += 1;
        if index != HOME {
            self.displacements[index] = waypoint.position - self.waypoints[HOME].position;
        }
    }
}

impl<A, S> Schedulable for Navigation<A, S>
where
    A: StaticData<Altitude>,
    S: StaticData<VelocityVector<f32, Meter>>,
{
    fn rate(&self) -> Rate {
        50
    }

    fn schedule(&mut self) -> bool {
        let rate = self.rate();
        let dt = 1.0 / rate as f32;

        if self.waypoints[HOME].position.latitude.0 == 0 {
            if let Some(position) = self.gnss.as_mut().map(|gnss| gnss.read(rate)).flatten() {
                self.waypoints[HOME].position = position;
            }
        }

        if self.waypoints[HOME].position.altitude.is_zero() {
            self.waypoints[HOME].position.altitude = self.altimeter.read();
        }
        let height = self.altimeter.read() - self.waypoints[HOME].position.altitude;
        let height = height.convert(|v| v as f32).to_unit(Meter);

        let gnss = self.gnss.as_mut().map(|gnss| gnss.read(rate)).flatten().map(|position| {
            (self.waypoints[HOME].position - position).convert(|v| v as f32).to_unit(Meter)
        });

        let velocity = self.speedometer.read();
        if let Some(position) = gnss {
            self.displacement.0 = self.filters[0].filter(position.x.value(), velocity.x.value());
            self.displacement.1 = self.filters[1].filter(position.y.value(), velocity.y.value());
        } else {
            self.displacement.0 += (velocity.x + (velocity.x - self.velocity.x) / 2.0).value() * dt;
            self.displacement.1 += (velocity.y + (velocity.y - self.velocity.y) / 2.0).value() * dt;
        }
        self.displacement.2 = self.filters[2].filter(height.value(), velocity.z.value());
        self.velocity = velocity;

        let s: DistanceVector<f32, Meter> = self.displacement.into();
        self.displacements[CURRENT] = s.convert(|v| v as i32);
        let waypoint = self.waypoints[self.next_waypoint as usize];
        let steerpoint = Steerpoint { index: self.next_waypoint, waypoint };
        let position = self.waypoints[HOME].position + self.displacements[CURRENT];
        self.output.write((position, steerpoint));
        true
    }
}
