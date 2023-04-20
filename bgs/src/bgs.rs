use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use byteorder::{BigEndian, WriteBytesExt};
use prost::Message;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use crate::bgs::protocol::session::v1::SessionService;
use crate::bgs::protocol::sns::v1::SocialNetworkService;

pub struct RemoteService {
    request_tx: UnboundedSender<Vec<u8>>,
    responders: Arc<Mutex<HashMap<u32, oneshot::Sender<(protocol::Header, Vec<u8>)>>>>
}

impl RemoteService {
    pub fn new(request_tx: UnboundedSender<Vec<u8>>) -> Self {
        Self {
            request_tx,
            responders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn rpc_no_response(&self, request_header: protocol::Header, request: impl Message) {
        let mut request_data_vec = vec![0; 2 + request_header.encoded_len() + request.encoded_len()];
        let mut request_data = request_data_vec.as_mut_slice();
        request_data.write_u16::<BigEndian>(request_header.encoded_len() as u16).unwrap();
        request_header.encode(&mut request_data).unwrap();
        request.encode(&mut request_data).unwrap();
        self.request_tx.send(request_data_vec).unwrap();
    }

    pub async fn rpc<R: Message + Default>(&self, request_header: protocol::Header, request: impl Message) -> (protocol::Header, R) {
        let token = request_header.token;
        self.rpc_no_response(request_header, request);

        let (response_tx, response_rx) = oneshot::channel();
        {
            let responders = self.responders.clone();
            let mut responders = responders.lock().unwrap();
            responders.insert(token, response_tx);
        }
        let (response_header, response) = response_rx.await.unwrap();
        (response_header, R::decode(response.as_slice()).unwrap())
    }

    pub async fn request(&self, request_header: protocol::Header, request: &[u8]) -> Option<Vec<u8>> {
        match request_header.service_hash.unwrap() {
            0 => {
                match request_header.method_id.unwrap() {
                    0 => {
                        let (response_header, response) = self.get_facebook_auth_code(request_header, Message::decode(request).unwrap()).await;
                        self.rpc_no_response(response_header, response)
                    }
                    _ => todo!()
                }
            }
            _ => todo!()
        }
        None
    }

    pub fn respond(&self, response_header: protocol::Header, response: Vec<u8>) {
        let responders = self.responders.clone();
        let mut responders = responders.lock().unwrap();
        if let Some(response_tx) = responders.remove(&response_header.token) {
            response_tx.send((response_header, response)).unwrap();
        }
    }
}

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/bgs.protocol.rs"));

    pub mod v2 {
        include!(concat!(env!("OUT_DIR"), "/bgs.protocol.v2.rs"));
    }

    pub mod account {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.account.v1.rs"));
        }
    }

    pub mod authentication {
        pub mod v1 {
            include!(concat!(
                env!("OUT_DIR"),
                "/bgs.protocol.authentication.v1.rs"
            ));
        }
    }

    pub mod challenge {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.challenge.v1.rs"));
        }
    }

    pub mod channel {
        include!(concat!(env!("OUT_DIR"), "/bgs.protocol.channel.rs"));

        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.channel.v1.rs"));
        }

        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.channel.v2.rs"));

            pub mod membership {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/bgs.protocol.channel.v2.membership.rs"
                ));
            }
        }
    }

    pub mod club {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.club.v1.rs"));

            pub mod membership {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/bgs.protocol.club.v1.membership.rs"
                ));
            }
        }
    }

    pub mod connection {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.connection.v1.rs"));
        }
    }

    pub mod diag {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.diag.v1.rs"));
        }
    }

    pub mod friends {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.friends.v1.rs"));
        }
    }

    pub mod game_utilities {
        pub mod v2 {
            pub mod client {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/bgs.protocol.game_utilities.v2.client.rs"
                ));
            }
        }
    }

    pub mod presence {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.presence.v1.rs"));
        }
    }

    pub mod report {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.report.v1.rs"));
        }

        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.report.v2.rs"));
        }
    }

    pub mod resources {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.resources.v1.rs"));
        }
    }

    pub mod session {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.session.v1.rs"));
        }
    }

    pub mod sns {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.sns.v1.rs"));
        }
    }

    pub mod user_manager {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.user_manager.v1.rs"));
        }
    }

    pub mod voice {
        pub mod v2 {
            pub mod client {
                include!(concat!(env!("OUT_DIR"), "/bgs.protocol.voice.v2.client.rs"));
            }
        }
    }

    pub mod whisper {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.whisper.v1.rs"));
        }
    }
}
