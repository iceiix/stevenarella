
use axis::Axis;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Invalid,
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn all() -> Vec<Direction> {
        vec![
            Direction::Down, Direction::Up,
            Direction::North, Direction::South,
            Direction::West, Direction::East,
        ]
    }

    pub fn from_string(val: &str) -> Direction {
        match val {
            "down" => Direction::Down,
            "up" => Direction::Up,
            "north" => Direction::North,
            "south" => Direction::South,
            "west" => Direction::West,
            "east" => Direction::East,
            _ => Direction::Invalid,
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            _ => unreachable!(),
        }
    }

    pub fn clockwise(&self) -> Direction {
        match *self {
            Direction::Down => Direction::Down,
            Direction::Up => Direction::Up,
            Direction::East => Direction::South,
            Direction::West => Direction::North,
            Direction::South => Direction::West,
            Direction::North => Direction::East,
            _ => unreachable!(),
        }
    }

    pub fn counter_clockwise(&self) -> Direction {
        match *self {
            Direction::Down => Direction::Down,
            Direction::Up => Direction::Up,
            Direction::East => Direction::North,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::North => Direction::West,
            _ => unreachable!(),
        }
    }

    pub fn get_offset(&self) -> (i32, i32, i32) {
        match *self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
            _ => unreachable!(),
        }
    }

    pub fn as_string(&self) -> &'static str {
        match *self {
            Direction::Down => "down",
            Direction::Up => "up",
            Direction::North => "north",
            Direction::South => "south",
            Direction::West => "west",
            Direction::East => "east",
            Direction::Invalid => "invalid",
        }
    }

    pub fn index(&self) -> usize {
        match *self {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
            _ => unreachable!(),
        }
    }

    pub fn horizontal_index(&self) -> usize {
        match *self {
            Direction::North => 2,
            Direction::South => 0,
            Direction::West => 1,
            Direction::East => 3,
            _ => unreachable!(),
        }
    }

    pub fn axis(&self) -> Axis {
        match *self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
            _ => unreachable!(),
        }
    }
}
