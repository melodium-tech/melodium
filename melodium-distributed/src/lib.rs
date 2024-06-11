mod messages;
mod protocol;

use async_std::net::{SocketAddr, TcpListener};
use melodium_common::descriptor::Version;
use messages::{ConfirmDistribution, Message};
use protocol::Protocol;

static VERSION: Version = Version::new(0, 1, 0);

pub async fn launch_listen(bind: SocketAddr, version: &Version) {
    let listener = TcpListener::bind(bind).await.unwrap();
    let (stream, _addr) = listener.accept().await.unwrap();

    let protocol = Protocol::new(stream);

    match protocol.recv_message().await {
        Ok(Message::AskDistribution(ask)) => {
            let accept = &ask.melodium_version == version && ask.distribution_version == VERSION;
            protocol
                .send_message(Message::ConfirmDistribution(ConfirmDistribution {
                    melodium_version: version.clone(),
                    distribution_version: VERSION.clone(),
                    accept,
                }))
                .await
                .unwrap();

            if !accept {
                return;
            }
        }
        _ => return,
    }

    let (collection, entrypoint, parameters) = match protocol.recv_message().await {
        Ok(Message::LoadAndLaunch(lal)) => (lal.collection, lal.entrypoint, lal.parameters),
        _ => return,
    };

    // Proceed to load of compiled elements

    // Proceed to design of the rest

    // Give it to engine

    // Manage engine calls to entrypoint
}
