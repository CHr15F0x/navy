//! UI is an overstatement here but let's assume it's sufficient for the exercise.
pub mod board;

use crate::state::Board;

const FIRST_LETTER: u8 = b'a';
const LAST_LETTER: u8 = FIRST_LETTER + Board::SIZE as u8 - 1;

/// Blocks until a valid peer idx is given
pub fn choose_peer(num_peers: usize) -> usize {
    tracing::info!("Choose peer!");

    let mut peer_idx: Option<usize> = None;

    while peer_idx.is_none() {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        peer_idx = match input.trim().parse::<usize>() {
            Ok(i) if i > 0 && i <= num_peers => Some(i - 1),
            Ok(_) | Err(_) => {
                tracing::warn!("Invalid peer chosen, try again");
                None
            }
        }
    }

    let peer_idx = peer_idx.expect("is some");

    tracing::trace!("Chosen peer: {}", peer_idx);

    peer_idx
}

/// Blocks until a valid coordinate is given in the form XY where:
/// - X is a letter from a to j and
/// - Y is a number from 1 to 10
pub fn shoot() -> (usize, usize) {
    let (x, y) = loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        let input_bytes = input.as_bytes();
        let x = match input_bytes.iter().next() {
            Some(x) if (FIRST_LETTER..=LAST_LETTER).any(|c| c == *x) => (x - FIRST_LETTER) as usize,
            Some(_) | None => {
                tracing::warn!("Invalid input, try again");
                continue;
            }
        };
        let y: usize = match input[1..].parse() {
            Ok(y) if y > 0 && y <= Board::SIZE => y,
            Ok(_) | Err(_) => {
                tracing::warn!("Invalid input, try again");
                continue;
            }
        };

        break (x, y - 1);
    };

    tracing::trace!("Shooting at: {x}, {y}");

    (x, y)
}
