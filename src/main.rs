use std::time::Duration;

use anyhow::Context;
use libp2p::PeerId;
use navy_lib::config;
use navy_lib::p2p;
use navy_lib::state::Board;
use navy_lib::state::Field;
use navy_lib::ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "navy=info");
    }

    tracing_subscriber::fmt::init();

    let i_am_boot = std::env::args().nth(1) == Some("-b".to_string());

    let mut my_board = config::read_my_board().context("Reading my board")?;
    let mut enemy_board = Board::default();

    let (_p2p_loop_handle, p2p_client) = p2p::start(i_am_boot)?;

    let game_loop_jh = tokio::spawn(async move {
        'outer: loop {
            enemy_board = choose_peer(p2p_client.clone(), my_board).await;

            // Play with peer
            // TODO decide who starts

            ui::board::draw(&my_board, &enemy_board);

            loop {
                println!("Your turn!");
                loop {
                    let (x, y) = ui::shoot().await;
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
                    let (x, y) = ui::shoot().await;
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

    tokio::select! {
        _ = game_loop_jh => {}
        _ = _p2p_loop_handle => {}
    }

    Ok(())
}

/// Get the index of the peer to play with.
async fn choose_peer(p2p_client: p2p::Client, _my_board: Board) -> Board {
    loop {
        let _peer = loop {
            let peers = wait_for_peers(p2p_client.clone()).await;
            ui::list_peers(&peers);
            let peer_idx = ui::choose_peer(peers.len()).await;
            if let Some(peer_idx) = peer_idx {
                break peers[peer_idx];
            }
        };

        // TODO send a game request to the peer
        break Board([Field::Ship, Field::Empty, Field::Empty, Field::Empty]);
    }
}

/// Wait for a random set of peers in the network to be discovered.
async fn wait_for_peers(p2p_client: p2p::Client) -> Vec<PeerId> {
    loop {
        let peers = p2p_client.get_peers().await;
        if !peers.is_empty() {
            break peers;
        }
        // Wait and retry
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
