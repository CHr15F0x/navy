use anyhow::Context;
use navy_lib::config;
use navy_lib::state::{Board, Cell};
use navy_lib::ui;

fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "navy=info");
    }

    tracing_subscriber::fmt::init();

    let boot_nodes = config::read_boot_nodes().context("Reading boot nodes")?;

    tracing::trace!(?boot_nodes);

    let mut my_board = config::read_my_board().context("Reading my board")?;
    let mut enemy_board = Board([Cell::Empty, Cell::Empty, Cell::Empty, Cell::Ship]);

    // TODO Connect to bootstrap nodes

    'outer: loop {
        // List peers and choose your opponent
        let num_peers = 10usize;

        let _peer_idx = ui::choose_peer(num_peers);

        // Play with peer
        // TODO decide who starts

        ui::board::draw(&my_board, &enemy_board);

        loop {
            tracing::info!("Your turn!");
            loop {
                let (x, y) = ui::shoot();
                match enemy_board.shoot(x, y) {
                    Some(_is_hit) => break,
                    None => {
                        tracing::warn!("Cell already targeted, try another location");
                        continue;
                    }
                }
            }

            // Render after my shot
            ui::board::draw(&my_board, &enemy_board);

            if enemy_board.all_sunk() {
                tracing::info!("You won!");
                break 'outer;
            }

            // Emulate the enemy's shot
            // TODO this should come from p2p
            tracing::info!("Enemy's turn!");
            loop {
                let (x, y) = ui::shoot();
                match my_board.shoot(x, y) {
                    Some(_is_hit) => break,
                    None => {
                        // TODO the enemy should check this condition locally ofc
                        tracing::warn!("Cell already targeted, try another location");
                        continue;
                    }
                }
            }

            // Render after enemy's shot
            ui::board::draw(&my_board, &enemy_board);

            if my_board.all_sunk() {
                tracing::info!("You lost!");
                break 'outer;
            }
        }
    }

    Ok(())
}
