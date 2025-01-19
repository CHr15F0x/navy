//! A blocking P2P client that can be used in the game loop.

use libp2p::PeerId;
use tokio::sync::mpsc::Sender;

use crate::p2p::Command;

#[derive(Clone)]
pub struct Client {
    cmd_tx: Sender<Command>,
}

impl Client {
    pub(crate) fn new(cmd_tx: Sender<Command>) -> Self {
        Client { cmd_tx }
    }

    /// Get closest peers.
    pub async fn get_peers(&self) -> Vec<PeerId> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.cmd_tx
            .send(Command::GetPeers(tx))
            .await
            .expect("Receiver not to be dropped");
        rx.await.expect("Sender not to be dropped")
    }
}
