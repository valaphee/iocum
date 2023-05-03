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
use staxmcje::{codec::Codec, packet::{c2s, s2c}, types::Intention, Decode, Encode, Error};

#[derive(Parser, Clone)]
struct Arguments {
    #[arg(long)]
    addr: String,
    #[arg(long)]
    remote_addr: Option<String>,

    #[arg(long)]
    access_token: String,
    #[arg(long)]
    selected_profile: Uuid,

    #[arg(long)]
    none: bool,
}

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    let listener = TcpListener::bind(arguments.addr.clone())
        .await
        .unwrap();

    loop {
        mitm(listener.accept().await.unwrap().0, arguments.clone()).await.unwrap();
    }
}

async fn mitm(
    socket: TcpStream,
    Arguments {
        addr,
        remote_addr,
        access_token,
        selected_profile,
        ..
    }: Arguments,
) -> staxmcje::Result<()> {
    let mut socket = Framed::new(socket, Codec::default());

    match next(&mut socket).await?.decode()? {
        c2s::HandshakePacket::Intention {
            protocol_version,
            host_name,
            port,
            intention,
        } => {
            let remote_addr = remote_addr.unwrap_or(format!("{host_name}:{port}"));
            if remote_addr == addr {
                return Err(Error::Unexpected)
            }
            let remote_socket = TcpStream::connect(remote_addr.clone()).await.unwrap();
            let mut remote_socket = Framed::new(remote_socket, Codec::default());

            let mut remote_addr_split = remote_addr.rsplitn(2, ':');
            encode_and_send(
                &mut remote_socket,
                &c2s::HandshakePacket::Intention {
                    protocol_version,
                    port: remote_addr_split.next().unwrap().parse().unwrap(),
                    host_name: remote_addr_split.next().unwrap().to_string(),
                    intention,
                },
            ).await;

            match intention {
                Intention::Status => {
                    let packet = next(&mut socket).await?.decode()?;
                    if matches!(packet, c2s::StatusPacket::StatusRequest { .. }) {
                        encode_and_send(&mut remote_socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }

                    let packet = next(&mut remote_socket).await?.decode()?;
                    if matches!(packet, s2c::StatusPacket::StatusResponse { .. }) {
                        encode_and_send(&mut socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }

                    let packet = next(&mut socket).await?.decode()?;
                    if matches!(packet, c2s::StatusPacket::PingRequest { .. }) {
                        encode_and_send(&mut remote_socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }

                    let packet = next(&mut remote_socket).await?.decode()?;
                    if matches!(packet, s2c::StatusPacket::PongResponse { .. }) {
                        encode_and_send(&mut socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }
                }
                Intention::Login => {
                    let packet = next(&mut socket).await?.decode()?;
                    if matches!(packet, c2s::LoginPacket::Hello { .. }) {
                        encode_and_send(&mut remote_socket, &packet).await;
                    } else {
                        return Err(Error::Unexpected);
                    }

                    match next(&mut remote_socket).await?.decode()? {
                        s2c::LoginPacket::Hello {
                            server_id,
                            public_key,
                            nonce,
                        } => {
                            let mut rng = OsRng::default();
                            let mut key = [0u8; 16];
                            rng.fill(&mut key);

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
                            remote_socket.codec_mut().enable_encryption(&key);

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
                                            println!("<< {:?}", packet);
                                            encode_and_send(&mut remote_socket, &packet).await;
                                        }
                                    }
                                    packet = next(&mut remote_socket) => {
                                        let packet = packet.unwrap();
                                        if let Ok(packet) = packet.decode::<s2c::GamePacket>() {
                                            println!(">> {:?}", packet);
                                            encode_and_send(&mut socket, &packet).await;
                                        }
                                    }
                                }
                            }
                        }
                        _ => return Err(Error::Unexpected),
                    };
                }
                _ => return Err(Error::Unexpected),
            }
        },
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
