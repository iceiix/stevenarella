
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Y,
    Z,
    X,
    None
}

impl Axis {
    pub fn as_string(&self) -> &'static str {
        match *self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
            Axis::None => "none",
        }
    }

    pub fn index(&self) -> usize {
        match *self {
            Axis::Y => 0,
            Axis::Z => 2,
            Axis::X => 1,
            Axis::None => 3,
        }
    }
}
