#![feature(result_flattening)]

use clap::Parser;
use flate2::Compression;
use futures::{SinkExt, StreamExt};
use rand::{rngs::OsRng, Rng};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};
use sha1::{digest::Update, Digest, Sha1};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use uuid::Uuid;

use mojang_session_api::{
    apis::{configuration::Configuration, default_api::join_server},
    models::JoinServerRequest,
};
use staxmcje::{
    codec::Codec,
    packet::{c2s, s2c},
    types::Intention,
    Decode, Encode, Error,
};

#[derive(Parser, Clone)]
#[command(about)]
struct Arguments {
    /// Address to bind to
    #[arg(long)]
    addr: String,
    /// Address to connect to
    #[arg(long)]
    remote_addr: Option<String>,
    /// Access token used for creating the session
    #[arg(long)]
    access_token: String,
    /// Selected profile used for creating the session
    #[arg(long)]
    selected_profile: Uuid,
}

#[tokio::main]
async fn main() {
    // parse arguments
    let arguments = Arguments::parse();
    // bind listener
    let listener = TcpListener::bind(arguments.addr.clone()).await.unwrap();
    // for each socket
    loop {
        let socket = listener.accept().await.unwrap().0;
        // spawn new task to handle socket
        let arguments = arguments.clone();
        tokio::task::spawn(async move {
            handle(socket, arguments).await.unwrap();
        });
    }
}

async fn handle(
    socket: TcpStream,
    Arguments {
        addr,
        remote_addr,
        access_token,
        selected_profile,
    }: Arguments,
) -> staxmcje::Result<()> {
    // create wrapped (client) socket
    let mut socket = Framed::new(socket, Codec::default());
    // receive intention packet
    match next(&mut socket).await?.decode()? {
        c2s::HandshakePacket::Intention {
            protocol_version,
            host_name,
            port,
            intention,
        } => {
            // either use the supplied remote address from arguments or use the original
            // address (transparent)
            let remote_addr = remote_addr.unwrap_or(format!("{host_name}:{port}"));
            if remote_addr == addr {
                return Err(Error::Unexpected);
            }
            // connect and created wrapped (server) socket
            let remote_socket = TcpStream::connect(remote_addr.clone()).await.unwrap();
            let mut remote_socket = Framed::new(remote_socket, Codec::default());
            // send intention packet
            {
                let mut remote_addr_split = remote_addr.rsplitn(2, ':');
                encode_and_send(
                    &mut remote_socket,
                    &c2s::HandshakePacket::Intention {
                        protocol_version,
                        host_name: remote_addr_split.next().unwrap().to_string(),
                        port: remote_addr_split.next().unwrap().parse().unwrap(),
                        intention,
                    },
                )
                .await;
            }
            // handle intention packet
            match intention {
                Intention::Status => {
                    // forward status request packet (1)
                    let packet = next(&mut socket).await?.decode()?;
                    if matches!(packet, c2s::StatusPacket::StatusRequest { .. }) {
                        encode_and_send(&mut remote_socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }
                    // forward status response packet (2)
                    let packet = next(&mut remote_socket).await?.decode()?;
                    if matches!(packet, s2c::StatusPacket::StatusResponse { .. }) {
                        encode_and_send(&mut socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }
                    // forward ping request packet (3)
                    let packet = next(&mut socket).await?.decode()?;
                    if matches!(packet, c2s::StatusPacket::PingRequest { .. }) {
                        encode_and_send(&mut remote_socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }
                    // forward ping response packet (4)
                    let packet = next(&mut remote_socket).await?.decode()?;
                    if matches!(packet, s2c::StatusPacket::PongResponse { .. }) {
                        encode_and_send(&mut socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }
                }
                Intention::Login => {
                    // forward c2s hello packet
                    let packet = next(&mut socket).await?.decode()?;
                    if matches!(packet, c2s::LoginPacket::Hello { .. }) {
                        encode_and_send(&mut remote_socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }
                    // receive s2c hello packet
                    let s2c::LoginPacket::Hello {
                        server_id,
                        public_key,
                        nonce,
                    } = next(&mut remote_socket).await?.decode()? else {
                        return Err(Error::Unexpected);
                    };
                    // generate random key
                    let mut rng = OsRng::default();
                    let mut key = [0u8; 16];
                    rng.fill(&mut key);
                    // send join server request to session server
                    join_server(
                        &Configuration::new(),
                        Some(JoinServerRequest {
                            access_token,
                            selected_profile,
                            server_id: hex::encode(
                                Sha1::new()
                                    .chain(server_id.as_bytes())
                                    .chain(key)
                                    .chain(&public_key)
                                    .finalize(),
                            ),
                        }),
                    )
                    .await
                    .unwrap();
                    // send c2s key packet
                    {
                        let public_key = RsaPublicKey::from_public_key_der(&public_key).unwrap();
                        encode_and_send(
                            &mut remote_socket,
                            &c2s::LoginPacket::Key {
                                key: public_key
                                    .encrypt(&mut rng, Pkcs1v15Encrypt::default(), &key)
                                    .unwrap(),
                                nonce: public_key
                                    .encrypt(&mut rng, Pkcs1v15Encrypt::default(), &nonce)
                                    .unwrap(),
                            },
                        )
                        .await;
                    }
                    // enable encryption
                    remote_socket.codec_mut().enable_encryption(&key);
                    // receive, handle and forward s2c login compression until game profile packet
                    // is received
                    loop {
                        match next(&mut remote_socket).await?.decode()? {
                            s2c::LoginPacket::LoginCompression {
                                compression_threshold,
                            } => {
                                remote_socket.codec_mut().enable_compression(
                                    Compression::default(),
                                    compression_threshold as u16,
                                );
                                encode_and_send(
                                    &mut socket,
                                    &s2c::LoginPacket::LoginCompression {
                                        compression_threshold,
                                    },
                                )
                                .await;
                                socket.codec_mut().enable_compression(
                                    Compression::default(),
                                    compression_threshold as u16,
                                );
                            }
                            packet => {
                                if matches!(packet, s2c::LoginPacket::GameProfile { .. }) {
                                    encode_and_send(&mut socket, &packet).await;
                                    break;
                                } else {
                                    return Err(Error::Unexpected);
                                }
                            }
                        }
                    }
                    loop {
                        tokio::select! {
                            packet = next(&mut socket) => {
                                let packet = packet.unwrap();
                                if let Ok(packet) = packet.decode::<c2s::GamePacket>() {
                                    println!("<<");
                                    println!("{:?}", packet);
                                    encode_and_send(&mut remote_socket, &packet).await;
                                }
                            }
                            packet = next(&mut remote_socket) => {
                                let packet = packet.unwrap();
                                if let Ok(packet) = packet.decode::<s2c::GamePacket>() {
                                    println!(">>");
                                    println!("{:?}", packet)
                                    encode_and_send(&mut socket, &packet).await;
                                }
                            }
                        }
                    }
                }
                _ => return Err(Error::Unexpected),
            }
        }
    }

    Ok(())
}

struct Packet(Vec<u8>);

impl Packet {
    fn decode<'a, T: Decode<'a>>(&'a self) -> staxmcje::Result<T> {
        T::decode(&mut self.0.as_slice())
    }
}

async fn encode_and_send(socket: &mut Framed<TcpStream, Codec>, packet: &impl Encode) {
    let mut data = vec![];
    packet.encode(&mut data).unwrap();
    socket.send(&data).await.unwrap();
}

async fn next(socket: &mut Framed<TcpStream, Codec>) -> staxmcje::Result<Packet> {
    socket
        .next()
        .await
        .ok_or(Error::UnexpectedEnd)
        .flatten()
        .map(Packet)
}
