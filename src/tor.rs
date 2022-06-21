use lightning::ln::peer_handler::SocketDescriptor;
//use lightning_net_tokio::SocketDescriptor;
//
use bitcoin::secp256k1::PublicKey;

use tokio::net::TcpStream;
use tokio::{io, time};
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};

use lightning::ln::peer_handler;
// use lightning::ln::peer_handler::SocketDescriptor as LnSocketTrait;
use lightning::ln::peer_handler::CustomMessageHandler;
use lightning::ln::msgs::{ChannelMessageHandler, RoutingMessageHandler, NetAddress};
use lightning::util::logger::Logger;

use std::task;
use std::net::SocketAddr;
use std::net::TcpStream as StdTcpStream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::hash::Hash;


// use prelude::*;

#[derive(Clone)]
pub struct TorDescriptor {
	id: u16,
	outbound_data: Arc<Mutex<Vec<u8>>>,
}
impl TorDescriptor {
	fn new() -> Self {
		// let id = conn.lock().unwrap().id;
		let id = 1;
		Self { outbound_data: Arc::new(Mutex::new(vec![])), id }
	}
}
impl PartialEq for TorDescriptor {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
impl Eq for TorDescriptor {}
impl core::hash::Hash for TorDescriptor {
	fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
		self.id.hash(hasher)
	}
}

impl SocketDescriptor for TorDescriptor {
	fn send_data(&mut self, data: &[u8], _resume_read: bool) -> usize {
		// self.outbound_data.lock().unwrap().extend_from_slice(data);
		println!("TOR tried to send data!!!!!");
		data.len()
	}

	fn disconnect_socket(&mut self) {
		println!("TOR tried to disconnect!!!!");
        }
}

pub async fn connect_outbound<CMH, RMH, L, UMH>(peer_manager: Arc<peer_handler::PeerManager<TorDescriptor, Arc<CMH>, Arc<RMH>, Arc<L>, Arc<UMH>>>, their_node_id: PublicKey, addr: SocketAddr) -> Option<impl std::future::Future<Output=()>> where
		CMH: ChannelMessageHandler + 'static + Send + Sync,
		RMH: RoutingMessageHandler + 'static + Send + Sync,
		L: Logger + 'static + ?Sized + Send + Sync,
		UMH: CustomMessageHandler + 'static + Send + Sync {
                    /*
	if let Ok(Ok(stream)) = time::timeout(Duration::from_secs(10), async { TcpStream::connect(&addr).await.map(|s| s.into_std().unwrap()) }).await {
		Some(setup_outbound(peer_manager, their_node_id, stream))
	} else { None }
                    */
		Some(setup_outbound(peer_manager, their_node_id, None))
}

pub fn setup_outbound<CMH, RMH, L, UMH>(peer_manager: Arc<peer_handler::PeerManager<TorDescriptor, Arc<CMH>, Arc<RMH>, Arc<L>, Arc<UMH>>>, their_node_id: PublicKey, stream: Option<StdTcpStream>) -> impl std::future::Future<Output=()> where
		CMH: ChannelMessageHandler + 'static + Send + Sync,
		RMH: RoutingMessageHandler + 'static + Send + Sync,
		L: Logger + 'static + ?Sized + Send + Sync,
		UMH: CustomMessageHandler + 'static + Send + Sync {
	// let remote_addr = get_addr_from_stream(&stream);
	// let (reader, mut write_receiver, read_receiver, us) = Connection::new(stream);
	#[cfg(debug_assertions)]
	// let last_us = Arc::clone(&us);
	let handle_opt = if let Ok(initial_send) = peer_manager.new_outbound_connection(their_node_id, TorDescriptor::new(), None) {
		Some(tokio::spawn(async move {
			// We should essentially always have enough room in a TCP socket buffer to send the
			// initial 10s of bytes. However, tokio running in single-threaded mode will always
			// fail writes and wake us back up later to write. Thus, we handle a single
			// std::task::Poll::Pending but still expect to write the full set of bytes at once
			// and use a relatively tight timeout.
			if let Ok(Ok(())) = tokio::time::timeout(Duration::from_millis(100), async {
				loop {
					match TorDescriptor::new().send_data(&initial_send, true) {
						v if v == initial_send.len() => break Ok(()),
						0 => {
							// write_receiver.recv().await;
							// In theory we could check for if we've been instructed to disconnect
							// the peer here, but its OK to just skip it - we'll check for it in
							// schedule_read prior to any relevant calls into RL.
						},
						_ => {
                                                    /*
							eprintln!("Failed to write first full message to socket!");
							peer_manager.socket_disconnected(&SocketDescriptor::new(Arc::clone(&us)));
                                                    */
							break Err(());
						}
					}
				}
			}).await {
				//Connection::schedule_read(peer_manager, us, reader, read_receiver, write_receiver).await;
			}
		}))
	} else {
		// Note that we will skip socket_disconnected here, in accordance with the PeerManager
		// requirements.
		None
	};

	async move {
		if let Some(handle) = handle_opt {
			if let Err(e) = handle.await {
				assert!(e.is_cancelled());
			} else {
				// This is certainly not guaranteed to always be true - the read loop may exit
				// while there are still pending write wakers that need to be woken up after the
				// socket shutdown(). Still, as a check during testing, to make sure tokio doesn't
				// keep too many wakers around, this makes sense. The race should be rare (we do
				// some work after shutdown()) and an error would be a major memory leak.
				//#[cfg(debug_assertions)]
				//assert!(Arc::try_unwrap(last_us).is_ok());
			}
		}
	}
}
