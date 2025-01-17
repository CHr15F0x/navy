//! A blocking P2P client that can be used in the game loop.

use tokio::sync::mpsc::Sender;

use crate::p2p::Command;

pub struct Client {
    cmd_tx: Sender<Command>,
}

impl Client {
    pub(crate) fn new(cmd_tx: Sender<Command>) -> Self {
        Client { cmd_tx }
    }

    /// Get a _printable_ representation of random closest peers.
    pub fn get_peers(&self) -> Vec<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.cmd_tx
            .blocking_send(Command::GetPeers(tx))
            .expect("Receiver not to be dropped");
        rx.blocking_recv()
            .expect("Sender not to be dropped")
            .into_iter()
            .map(|peer_id| {
                if peer_id == *crate::p2p::BOOT_PEER_ID {
                    peer_id.to_string() + " (BOOT)"
                } else {
                    peer_id.to_string()
                }
            })
            .collect()
    }
}
