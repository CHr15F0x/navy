use libp2p::identity::Keypair;
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{identify, ping};

/// I'm not using mDNS because I'd like this behavior to somewhat reflect a more real setup
/// even though it's run in a local network. Hence kad as a discovery method.
/// Nevertheless I'm skipping autonat, dcutr, and the relay transport for simplicity.
#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub ping: ping::Behaviour,
}

impl Behaviour {
    pub fn new(keypair: &Keypair) -> Self {
        let local_public_key = keypair.public();
        let local_peer_id = local_public_key.to_peer_id();

        Behaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                identify::PROTOCOL_NAME.to_string(),
                local_public_key,
            )),
            kademlia: kad::Behaviour::new(local_peer_id, MemoryStore::new(local_peer_id)),
            ping: ping::Behaviour::new(ping::Config::new()),
        }
    }
}
