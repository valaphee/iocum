use std::{fs::File, sync::Arc};

use byteorder::{BigEndian, ReadBytesExt};
use clap::{arg, Parser};
use futures_util::{SinkExt, StreamExt};
use rsa::pkcs1v15::SigningKey;
use rsa::RsaPrivateKey;
use rsa::signature::RandomizedSigner;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, handshake::server, Message};
use url::Url;

use tlsbogie::ResolvesServerCertAutogen;

#[derive(Parser)]
enum Arguments {
    Mitm {
        remote_addr: Url,
        #[arg(long)]
        local_addr: Option<Url>,
        #[arg(long)]
        default_sni: Option<String>,
    },
    Patch {
        file: File,
    },
}

#[tokio::main]
async fn main() {
    match Arguments::parse() {
        Arguments::Mitm {
            remote_addr,
            local_addr,
            default_sni,
        } => {
            let local_addr = local_addr.unwrap();
            let listener = TcpListener::bind(
                local_addr
                    .socket_addrs(|| Some(1119))
                    .unwrap()
                    .first()
                    .unwrap(),
            )
            .await
            .unwrap();

            // setup tls server config
            let tls_server_config = ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(ResolvesServerCertAutogen::new(
                    "certs",
                    default_sni.unwrap_or(local_addr.host().unwrap().to_string()),
                )));
            let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_config));

            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let tls_acceptor = tls_acceptor.clone();
                let remote_addr = remote_addr.clone();

                tokio::task::spawn(async move {
                    let stream = tls_acceptor.accept(stream).await.unwrap();
                    let mut stream = tokio_tungstenite::accept_hdr_async(
                        stream,
                        |request: &server::Request, mut response: server::Response| {
                            println!("----BEGIN HTTP REQUEST-----");
                            println!(
                                "{} {} {:?}",
                                request.method(),
                                request.uri().path_and_query().unwrap().as_str(),
                                request.version()
                            );
                            for (header_name, header_value) in request.headers().iter() {
                                println!("{}: {}", header_name, header_value.to_str().unwrap());
                            }
                            println!("----END HTTP REQUEST-----");

                            response.headers_mut().append(
                                "sec-websocket-protocol",
                                "v1.rpc.battle.net".parse().unwrap(),
                            );

                            Ok(response)
                        },
                    )
                    .await
                    .unwrap();

                    let mut request = remote_addr.into_client_request().unwrap();
                    request.headers_mut().append(
                        "sec-websocket-protocol",
                        "v1.rpc.battle.net".parse().unwrap(),
                    );
                    let (mut remote_stream, response) =
                        tokio_tungstenite::connect_async_tls_with_config(request, None, None)
                            .await
                            .unwrap();
                    println!("----BEGIN HTTP RESPONSE-----");
                    println!("{:?} {}", response.version(), response.status().as_str());
                    for (header_name, header_value) in response.headers().iter() {
                        println!("{}: {}", header_name, header_value.to_str().unwrap());
                    }
                    println!("----END HTTP RESPONSE-----");

                    loop {
                        tokio::select! {
                            message = stream.next() => {
                                remote_stream.send(message.unwrap().unwrap()).await.unwrap();
                            }
                            message = remote_stream.next() => {
                                stream.send(message.unwrap().unwrap()).await.unwrap();
                            }
                        }
                    }
                });
            }
        }
        Arguments::Patch { file } => {
            let file_content = std::fs::read_to_string(file).unwrap();
            if let Some(certificate_bundle_start) = file_content.find("{\"Created\":\"") {
                serde_json::to_string(&CertificateBundle {
                    created: 0,
                    certificates: vec![],
                    public_keys: vec![],
                    signing_certificates: vec![],
                    root_ca_public_keys: vec![],
                }).unwrap();

                let mut rng = rand::thread_rng();
                let key_pair = RsaPrivateKey::new(&mut rng, 2048).unwrap();
                let signing_key = SigningKey::<Sha256>::new_with_prefix(private_key);
                let sign = signing_key.sign_with_rng(&mut rng, b"");
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CertificateBundle {
    created: u64,
    certificates: Vec<CertificateBundlePublicKey>,
    public_keys: Vec<CertificateBundlePublicKey>,
    signing_certificates: Vec<CertificateBundleCertificate>,
    root_ca_public_keys: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CertificateBundlePublicKey {
    uri: String,
    #[serde(with = "hex")]
    sha256: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct CertificateBundleCertificate {
    data: String
}
