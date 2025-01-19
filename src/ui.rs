//! UI is an overstatement here but let's assume it's sufficient for the exercise.
//!
//! The following functions were initially blocking, since tokio's io::stdin is
//! still blocking under the hood and tokio's docs don't recommend using it for user
//! interaction, but then it's a hassle to interleave stdin interaction with async
//! events from p2p, in particular, when we haven't chosen an opponent yet and
//! somebody has just offered us a game.
pub mod board;

use libp2p::PeerId;
use tokio::io::AsyncBufReadExt;

use crate::{p2p, state::Board};

const FIRST_LETTER: u8 = b'a';
const LAST_LETTER: u8 = FIRST_LETTER + Board::SIZE as u8 - 1;

pub fn list_peers(peers: &[PeerId]) {
    println!("Peers:");
    for (i, peer) in peers.iter().enumerate() {
        if *peer == *p2p::BOOT_PEER_ID {
            println!("{}. {} (BOOT)", i + 1, peer);
        } else {
            println!("{}. {}", i + 1, peer);
        }
    }
}

/// Blocks until a valid peer idx is given, returns None if 'r' is given
pub async fn choose_peer(num_peers: usize) -> Option<usize> {
    println!("Choose peer, or 'r' to retry!");

    let mut peer_idx: Option<usize> = None;

    while peer_idx.is_none() {
        let input = input().await;

        if input.trim() == "r" {
            return None;
        }

        peer_idx = match input.trim().parse::<usize>() {
            Ok(i) if i > 0 && i <= num_peers => Some(i - 1),
            Ok(_) | Err(_) => {
                println!("Invalid peer chosen, try again");
                None
            }
        }
    }

    peer_idx
}

/// Blocks until a valid coordinate is given in the form XY where:
/// - X is a letter from a to j and
/// - Y is a number from 1 to 10
pub async fn shoot() -> (usize, usize) {
    let (x, y) = loop {
        let input = input().await;
        let input = input.trim();
        let input_bytes = input.as_bytes();
        let x = match input_bytes.iter().next() {
            Some(x) if (FIRST_LETTER..=LAST_LETTER).any(|c| c == *x) => (x - FIRST_LETTER) as usize,
            Some(_) | None => {
                println!("Invalid input, try again");
                continue;
            }
        };
        let y: usize = match input[1..].parse() {
            Ok(y) if y > 0 && y <= Board::SIZE => y,
            Ok(_) | Err(_) => {
                println!("Invalid input, try again");
                continue;
            }
        };

        break (x, y - 1);
    };

    tracing::trace!("Shooting at: {x}, {y}");

    (x, y)
}

/// Reads a line from stdin, if a non-UTF-8 byte is encountered, the read is retried.
pub async fn input() -> String {
    let mut input = String::new();
    loop {
        if tokio::io::BufReader::new(tokio::io::stdin())
            .read_line(&mut input)
            .await
            .is_ok()
        {
            break;
        }
    }

    input
}

pub async fn wait_for_user_yn() -> bool {
    loop {
        let input = input().await;
        match input.trim() {
            "y" => return true,
            "n" => return false,
            _ => println!("[y/n]?"),
        }
    }
}
