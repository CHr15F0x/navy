use crate::state::{Board, Cell};
use std::io::Read;

use anyhow::Context;

const MY_BOARD_FILE: &str = "my.board";
// At most 2 newline marker bytes, so we're "future proof"
const UPPER_BOARD_FILE_LIMIT: usize = (Board::SIZE + 2) * Board::SIZE + 2;
// No newline at the eof
const LOWER_BOARD_FILE_LIMIT: usize = (Board::SIZE + 1) * Board::SIZE;

pub fn read_my_board() -> anyhow::Result<Board> {
    let mut board = Board::default();

    let file = std::io::BufReader::new(std::fs::File::open(MY_BOARD_FILE)?);

    let mut file = file.take(
        UPPER_BOARD_FILE_LIMIT
            .try_into()
            .expect("board size plus some slack always fits into u64"),
    );

    let mut buf = String::new();
    let num_read_bytes = file
        .read_to_string(&mut buf)
        .context("Reading board file")?;

    if num_read_bytes < LOWER_BOARD_FILE_LIMIT {
        return Err(anyhow::anyhow!("Board file is incomplete"));
    }

    let actual_line_count = buf.lines().count();
    if actual_line_count != Board::SIZE {
        return Err(anyhow::anyhow!(
            "Invalid number of lines in the board file: {}, expected: {}",
            actual_line_count,
            Board::SIZE
        ));
    }

    for (y, line) in buf.lines().enumerate() {
        if line.len() != Board::SIZE {
            return Err(anyhow::anyhow!(
                "Invalid line {} length in the board file",
                y + 1
            ));
        }

        for (x, c) in line.chars().enumerate() {
            board.set(x, y, Cell::try_from(c)?);
        }
    }

    anyhow::ensure!(
        board.validate(),
        "Board has invalid number of ships, expected: {}",
        Board::NUM_SHIPS
    );

    Ok(board)
}

impl TryFrom<char> for Cell {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Cell::Empty),
            '+' => Ok(Cell::Ship),
            _ => Err(anyhow::anyhow!(
                "Invalid character: {}, only empty '.' or ship '+' are allowed",
                c
            )),
        }
    }
}
