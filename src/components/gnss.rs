use crate::datastructures::coordinate::Position;

pub trait GNSS {
    fn get_position(&self) -> Position;
}
