//! Game state

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Board(pub [Field; Board::SIZE * Board::SIZE]);

impl Board {
    pub const SIZE: usize = 2;
    pub const NUM_SHIPS: usize = 1;

    /// Panics if x or y are out of bounds
    pub fn get(&self, x: usize, y: usize) -> Field {
        self.0[y * Self::SIZE + x]
    }

    /// Panics if x or y are out of bounds
    pub fn set(&mut self, x: usize, y: usize, cell: Field) {
        self.0[y * Self::SIZE + x] = cell;
    }

    /// Validate initial board state:
    /// - There must be exactly NUM_SHIPS ships
    pub fn validate(&self) -> bool {
        self.0.iter().fold(0, |num_ships, cell| {
            if *cell == Field::Ship {
                num_ships + 1
            } else {
                num_ships
            }
        }) == Self::NUM_SHIPS
    }

    pub fn all_sunk(&self) -> bool {
        !self.0.iter().any(|cell| *cell == Field::Ship)
    }

    /// Panics if x or y are out of bounds
    ///
    /// Returns:
    /// - Some(true) if it's a hit,
    /// - Some(false) if it's a miss,
    /// - None if the cell was already targeted.
    pub fn shoot(&mut self, x: usize, y: usize) -> Option<bool> {
        match self.get(x, y) {
            Field::Empty => {
                self.set(x, y, Field::Miss);
                Some(false)
            }
            Field::Ship => {
                self.set(x, y, Field::Hit);
                Some(true)
            }
            Field::Hit | Field::Miss => None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Field {
    #[default]
    Empty,
    Ship,
    Hit,
    Miss,
}
