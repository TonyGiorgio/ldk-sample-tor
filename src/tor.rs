use bitcoin::secp256k1::PublicKey;
use lightning_net_tokio::SocketDescriptor;

use lightning::ln::msgs::{ChannelMessageHandler, RoutingMessageHandler};
use lightning::ln::peer_handler;
use lightning::ln::peer_handler::CustomMessageHandler;
use lightning::util::logger::Logger;
use tor::{TorService, TorServiceParam};
use tor_stream::{ToTargetAddr, TorStream};

use std::convert::TryInto;
use std::sync::Arc;

pub fn create_tor_service() -> TorService {
	let socks_port: u16 = 19054;
	TorServiceParam {
		socks_port: Some(socks_port),
		data_dir: String::from("/tmp/sifir_rs_sdk/"),
		bootstrap_timeout_ms: Some(45000),
	}
	.try_into()
	.unwrap()
}

pub async fn connect_outbound<CMH, RMH, L, UMH>(
	peer_manager: Arc<
		peer_handler::PeerManager<SocketDescriptor, Arc<CMH>, Arc<RMH>, Arc<L>, Arc<UMH>>,
	>,
	their_node_id: PublicKey, addr: impl ToTargetAddr,
) -> Option<impl std::future::Future<Output = ()>>
where
	CMH: ChannelMessageHandler + 'static + Send + Sync,
	RMH: RoutingMessageHandler + 'static + Send + Sync,
	L: Logger + 'static + ?Sized + Send + Sync,
	UMH: CustomMessageHandler + 'static + Send + Sync,
{
	let stream = TorStream::connect_with_address("127.0.0.1:19054".parse().unwrap(), addr)
		.expect("Failed to connect");
	let tcp_stream = stream.into_inner();

	Some(lightning_net_tokio::setup_outbound(peer_manager, their_node_id, tcp_stream))
}
