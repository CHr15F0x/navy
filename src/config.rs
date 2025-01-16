use crate::state::{Board, Cell};
use std::fs::read_to_string;
use std::io::Read;

use anyhow::Context;

const BOOT_NODES_FILE: &str = "boot.nodes";
// Tbh 1 boot node should be fine
const MY_BOARD_FILE: &str = "my.board";
// At most 2 newline marker bytes, so we're "future proof"
const UPPER_BOARD_FILE_LIMIT: usize = (Board::SIZE + 2) * Board::SIZE + 2;
// No newline at the eof
const LOWER_BOARD_FILE_LIMIT: usize = (Board::SIZE + 1) * Board::SIZE;

pub fn read_boot_nodes() -> anyhow::Result<Vec<String>> {
    // TODO this should be limited in the number of bytes ingested and in the number of boot node addresses produced
    let boot_nodes = read_to_string(BOOT_NODES_FILE)
        .context("Reading boot nodes file")?
        .lines()
        .map(String::from)
        .collect::<Vec<_>>();

    anyhow::ensure!(
        !boot_nodes.is_empty(),
        "At least one bootstrap node is required in {}",
        BOOT_NODES_FILE
    );

    Ok(boot_nodes)
}

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
