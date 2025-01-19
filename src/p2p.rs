use std::{collections::HashMap, sync::LazyLock, time::Duration};

use base64::Engine;
use libp2p::{
    futures::StreamExt,
    identify,
    identity::Keypair,
    kad::{self, GetClosestPeersOk, QueryId},
    swarm::SwarmEvent,
    Multiaddr, PeerId, Swarm, SwarmBuilder,
};

mod behaviour;
mod client;

use behaviour::{Behaviour, BehaviourEvent};
pub use client::Client;

/// protobuf+Base64 encoded private key of an example boot node, just for the example, never do this in production.
static BOOT_KEYPAIR: LazyLock<Keypair> = LazyLock::new(|| {
    let buf = base64::engine::general_purpose::STANDARD
        .decode("CAESQC9T5dUHWfbZE/eHE9zPeHVkrPFjn/73BK06LyCR4PWtGZH9dCIm1fLISYzSE9DvSJi02NWJSkhT7C1tzvaLH1E=")
        .expect("Valid private key");
    Keypair::from_protobuf_encoding(&buf).expect("Valid keypair")
});
pub static BOOT_PEER_ID: LazyLock<PeerId> = LazyLock::new(|| BOOT_KEYPAIR.public().to_peer_id());

const TEN_MINUTES: u32 = 10 * 60 * 1000;

pub fn start(i_am_boot: bool) -> anyhow::Result<(tokio::task::JoinHandle<()>, Client)> {
    let keypair = if i_am_boot {
        BOOT_KEYPAIR.clone()
    } else {
        Keypair::generate_ed25519()
    };

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_quic_config(|mut config| {
            config.max_idle_timeout = TEN_MINUTES;
            config
        })
        .with_behaviour(|keypair| Ok(Behaviour::new(keypair)))?
        .with_swarm_config(|config| {
            config.with_idle_connection_timeout(Duration::from_secs(TEN_MINUTES.into()))
        })
        .build();

    if i_am_boot {
        swarm.listen_on("/ip4/0.0.0.0/udp/50000/quic-v1".parse()?)?;
    } else {
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        // And then dial the boot node
        swarm.dial("/ip4/127.0.0.1/udp/50000/quic-v1".parse::<Multiaddr>()?)?;
    }

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(1);

    let jh = tokio::spawn(async move {
        let mut pending_queries = HashMap::new();

        loop {
            tokio::select! {
                command = cmd_rx.recv() => {
                    handle_command(&mut swarm, &mut pending_queries, command.expect("Command sender not to be dropped"));
                }
                event = swarm.select_next_some() => {
                    handle_event(&mut swarm, &mut pending_queries, event).await;
                }
            }
        }
    });

    Ok((jh, Client::new(cmd_tx)))
}

type GetPeersReplyTx = tokio::sync::oneshot::Sender<Vec<libp2p::PeerId>>;

pub(crate) enum Command {
    GetPeers(GetPeersReplyTx),
}

fn handle_command(
    swarm: &mut Swarm<Behaviour>,
    pending_queries: &mut HashMap<QueryId, GetPeersReplyTx>,
    command: Command,
) {
    match command {
        Command::GetPeers(tx) => {
            // Alternatively we could assume that bootstaping is good enough,
            // and just iterate the local kbuckets that contain any peers.
            // The disadvantage is that with traditional bootstrap we loose
            // the ability to randomize the search for the peers, because bootstrap
            // always looks up our own peer id.
            let query_id = swarm
                .behaviour_mut()
                .kademlia
                .get_closest_peers(PeerId::random());
            pending_queries.insert(query_id, tx);
        }
    }
}

async fn handle_event(
    swarm: &mut Swarm<Behaviour>,
    pending_queries: &mut HashMap<QueryId, GetPeersReplyTx>,
    event: SwarmEvent<BehaviourEvent>,
) {
    match event {
        SwarmEvent::Behaviour(behaviour::BehaviourEvent::Identify(identify::Event::Received {
            peer_id,
            info:
                identify::Info {
                    listen_addrs,
                    observed_addr,
                    ..
                },
            ..
        })) => {
            {
                // We're using the default kad protocol name for now, which wouldn't be good if we
                // wanted to reach outside the local network, as most probably we'd like to have our own DHT,
                // otherwise we're polluting the ipfs' DHT.
                // On the other hand, I imagine we could take advantage of the default ipfs DHT to circumvent
                // the lack of real boot nodes but then we'd have to add an additional mechanism
                // to discover the "real" navy nodes, for example by providing a specific key in the ipfs' DHT
                // that means "I'm a navy node".
                swarm.add_external_address(observed_addr);

                for addr in &listen_addrs {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                }
            }
        }
        SwarmEvent::Behaviour(behaviour::BehaviourEvent::Kademlia(
            // TODO check if the last result accumulates intermediate results
            kad::Event::OutboundQueryProgressed {
                id,
                result: kad::QueryResult::GetClosestPeers(inner_result),
                step,
                ..
            },
        )) if step.last => {
            // TODO check if the last result accumulates intermediate results
            let tx = pending_queries
                .remove(&id)
                .expect("Query id to be in the pending queries");
            let peers = match inner_result {
                Ok(GetClosestPeersOk { peers, .. }) => {
                    peers.into_iter().map(|p| p.peer_id).collect()
                }
                Err(_) => Vec::new(),
            };
            tx.send(peers).expect("Receiver not to be dropped");
        }
        _ => {
            tracing::trace!(?event);
        }
    }
}
