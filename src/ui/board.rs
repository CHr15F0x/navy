use crate::state::{Board, Cell};
use std::fmt;

pub fn draw(my_board: &Board, enemy_board: &Board) {
    println!("Enemy:\n{}", EnemyBoard(enemy_board));
    println!("Me:\n{my_board}");
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, "."),
            Cell::Ship => write!(f, "+"),
            Cell::Hit => write!(f, "x"),
            Cell::Miss => write!(f, "o"),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..Board::SIZE {
            writeln!(
                f,
                "{}",
                Row(&self.0[y * Board::SIZE..(y + 1) * Board::SIZE])
            )?;
        }

        Ok(())
    }
}

struct Row<'a>(&'a [Cell]);

impl fmt::Display for Row<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.0.len() {
            write!(f, "{}", self.0[i])?;
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
                    Cell::Empty | Cell::Ship => write!(f, ".")?,
                    Cell::Hit => write!(f, "x")?,
                    Cell::Miss => write!(f, "o")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
