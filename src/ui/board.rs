use crate::state::{Board, Field};
use std::fmt;

pub fn draw(my_board: &Board, enemy_board: &Board) {
    println!("Enemy:\n{}", EnemyBoard(enemy_board));
    println!("Me:\n{my_board}");
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..Board::SIZE {
            for x in 0..Board::SIZE {
                match self.get(x, y) {
                    Field::Empty => write!(f, ".")?,
                    Field::Ship => write!(f, "+")?,
                    Field::Hit => write!(f, "x")?,
                    Field::Miss => write!(f, "o")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

struct EnemyBoard<'a>(&'a Board);

impl fmt::Display for EnemyBoard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..Board::SIZE {
            for x in 0..Board::SIZE {
                match self.0.get(x, y) {
                    Field::Empty | Field::Ship => write!(f, ".")?,
                    Field::Hit => write!(f, "x")?,
                    Field::Miss => write!(f, "o")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
