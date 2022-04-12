use crate::types::{
    coordinate::Position,
    measurement::{unit, Displacement, VelocityVector, ENU},
};

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct INS {
    pub velocity_vector: VelocityVector<f32, unit::Ms, ENU>,
    pub position: Position,
    pub displacement: Displacement<f32, unit::Meter, ENU>,
}
