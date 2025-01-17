use std::time::Duration;

use anyhow::{anyhow, Context};
use navy_lib::config;
use navy_lib::p2p;
use navy_lib::state::{Board, Cell};
use navy_lib::ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "navy=info");
    }

    tracing_subscriber::fmt::init();

    let i_am_boot = std::env::args().nth(1) == Some("-b".to_string());

    let mut my_board = config::read_my_board().context("Reading my board")?;
    let mut enemy_board = Board([Cell::Empty, Cell::Empty, Cell::Empty, Cell::Ship]);

    let (_p2p_loop_handle, p2p_client) = p2p::start(i_am_boot)?;

    let game_loop_jh = std::thread::spawn(move || {
        'outer: loop {
            let _peer_idx = choose_peer(&p2p_client);

            // Play with peer
            // TODO decide who starts

            ui::board::draw(&my_board, &enemy_board);

            loop {
                println!("Your turn!");
                loop {
                    let (x, y) = ui::shoot();
                    match enemy_board.shoot(x, y) {
                        Some(_is_hit) => break,
                        None => {
                            println!("Cell already targeted, try another location");
                            continue;
                        }
                    }
                }

                // Render after my shot
                ui::board::draw(&my_board, &enemy_board);

                if enemy_board.all_sunk() {
                    println!("You won!");
                    break 'outer;
                }

                // Emulate the enemy's shot
                // TODO this should come from p2p
                println!("Enemy's turn!");
                loop {
                    let (x, y) = ui::shoot();
                    match my_board.shoot(x, y) {
                        Some(_is_hit) => break,
                        None => {
                            // TODO the enemy should check this condition locally ofc
                            println!("Cell already targeted, try another location");
                            continue;
                        }
                    }
                }

                // Render after enemy's shot
                ui::board::draw(&my_board, &enemy_board);

                if my_board.all_sunk() {
                    println!("You lost!");
                    break 'outer;
                }
            }
        }
    });

    game_loop_jh
        .join()
        .map_err(|_| anyhow!("Game loop thread panicked"))?;

    Ok(())
}

/// Get the index of the peer to play with.
fn choose_peer(p2p_client: &p2p::Client) -> usize {
    loop {
        let peers = wait_for_peers(p2p_client);
        ui::list_peers(&peers);
        let peer_idx = ui::choose_peer(peers.len());
        if let Some(peer_idx) = peer_idx {
            break peer_idx;
        }
    }
}

/// Wait for a random set of peers in the network to be discovered.
fn wait_for_peers(p2p_client: &p2p::Client) -> Vec<String> {
    loop {
        let peers = p2p_client.get_peers();
        if !peers.is_empty() {
            break peers;
        }
        // Wait and retry
        std::thread::sleep(Duration::from_secs(1));
    }
}
