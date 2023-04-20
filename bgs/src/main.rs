use std::{io::Write, path::PathBuf, sync::Arc, time::SystemTime};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use clap::{arg, Parser};
use futures_util::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use openssl::{sha::sha256, x509::X509};
use prost::Message;
use rsa::{
    pkcs1v15::SigningKey,
    pkcs8::DecodePublicKey,
    signature::{RandomizedSigner, SignatureEncoding},
    PublicKeyParts, RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::{net::TcpListener, sync::mpsc::unbounded_channel};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_tungstenite::{
    tungstenite::{client::IntoClientRequest, handshake::server},
    Connector,
};
use url::Url;

use staxtls::ResolvesServerCertAutogen;

mod bgs;

#[derive(Parser)]
enum Arguments {
    Mitm {
        remote_uri: Url,
        #[arg(long)]
        local_uri: Option<Url>,

        #[arg(long)]
        default_sni: Option<String>,
    },
    Patch {
        file: PathBuf,
        uris: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    match Arguments::parse() {
        Arguments::Mitm {
            remote_uri,
            local_uri,
            default_sni,
        } => {
            let local_uri = local_uri.unwrap();
            let listener = TcpListener::bind(
                local_uri
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
                    default_sni.unwrap_or(local_uri.host().unwrap().to_string()),
                )));
            let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_config));

            loop {
                let (stream, address) = listener.accept().await.unwrap();
                let tls_acceptor = tls_acceptor.clone();
                let remote_uri = remote_uri.clone();

                tokio::task::spawn(async move {
                    let stream = tls_acceptor.accept(stream).await.unwrap();
                    let mut stream = tokio_tungstenite::accept_hdr_async(
                        stream,
                        |request: &server::Request, mut response: server::Response| {
                            println!("<< {address}");
                            println!(
                                "{} {} {:?}",
                                request.method(),
                                request.uri().path_and_query().unwrap().as_str(),
                                request.version()
                            );
                            for (header_name, header_value) in request.headers().iter() {
                                println!("{}: {}", header_name, header_value.to_str().unwrap());
                            }

                            response.headers_mut().append(
                                "sec-websocket-protocol",
                                "v1.rpc.battle.net".parse().unwrap(),
                            );

                            Ok(response)
                        },
                    )
                    .await
                    .unwrap();

                    let mut request = remote_uri.into_client_request().unwrap();
                    request.headers_mut().append(
                        "sec-websocket-protocol",
                        "v1.rpc.battle.net".parse().unwrap(),
                    );
                    let (mut remote_stream, response) =
                        tokio_tungstenite::connect_async_tls_with_config(
                            request,
                            None,
                            Some(Connector::NativeTls(
                                TlsConnector::builder()
                                    .danger_accept_invalid_hostnames(true)
                                    .build()
                                    .unwrap(),
                            )),
                        )
                        .await
                        .unwrap();
                    println!(">> {address}");
                    println!("{:?} {}", response.version(), response.status().as_str());
                    for (header_name, header_value) in response.headers().iter() {
                        println!("{}: {}", header_name, header_value.to_str().unwrap());
                    }

                    let (client_message_tx, mut client_message_rx) = unbounded_channel();
                    let client_service =
                        Arc::new(bgs::RemoteService::new(client_message_tx.clone()));

                    let (server_message_tx, mut server_message_rx) = unbounded_channel();
                    let server_service =
                        Arc::new(bgs::RemoteService::new(server_message_tx.clone()));

                    let client_service_2 = client_service.clone();
                    let (server_request_tx, mut server_request_rx) =
                        unbounded_channel::<(bgs::protocol::Header, Vec<u8>)>();
                    tokio::spawn(async move {
                        loop {
                            while let Some(request) = server_request_rx.recv().await {
                                if let Some(response) = client_service_2
                                    .handle_request(
                                        request.0.service_hash.unwrap(),
                                        request.0.method_id.unwrap(),
                                        &request.1,
                                    )
                                    .await
                                {
                                    let header = bgs::protocol::Header {
                                        service_id: 254,
                                        method_id: None,
                                        token: request.0.token,
                                        object_id: None,
                                        size: Some(response.len() as u32),
                                        status: Some(0),
                                        error: vec![],
                                        timeout: None,
                                        is_response: None,
                                        forward_targets: vec![],
                                        service_hash: None,
                                        client_id: None,
                                        fanout_target: vec![],
                                        client_id_fanout_target: vec![],
                                        client_record: None,
                                        original_sender: None,
                                        sender_token: None,
                                        router_label: None,
                                        error_reason: None,
                                    };

                                    let mut packet_vec =
                                        vec![0; 2 + header.encoded_len() + response.len()];
                                    let mut packet = packet_vec.as_mut_slice();
                                    packet
                                        .write_u16::<BigEndian>(header.encoded_len() as u16)
                                        .unwrap();
                                    header.encode(&mut packet).unwrap();
                                    packet.write(&response).unwrap();
                                    server_message_tx.send(packet_vec).unwrap();
                                }
                            }
                        }
                    });

                    let server_service_2 = server_service.clone();
                    let (client_request_tx, mut client_request_rx) =
                        unbounded_channel::<(bgs::protocol::Header, Vec<u8>)>();
                    tokio::spawn(async move {
                        loop {
                            while let Some(request) = client_request_rx.recv().await {
                                if let Some(response) = server_service_2
                                    .handle_request(
                                        request.0.service_hash.unwrap(),
                                        request.0.method_id.unwrap(),
                                        &request.1,
                                    )
                                    .await
                                {
                                    let header = bgs::protocol::Header {
                                        service_id: 254,
                                        method_id: None,
                                        token: request.0.token,
                                        object_id: None,
                                        size: Some(response.len() as u32),
                                        status: Some(0),
                                        error: vec![],
                                        timeout: None,
                                        is_response: None,
                                        forward_targets: vec![],
                                        service_hash: None,
                                        client_id: None,
                                        fanout_target: vec![],
                                        client_id_fanout_target: vec![],
                                        client_record: None,
                                        original_sender: None,
                                        sender_token: None,
                                        router_label: None,
                                        error_reason: None,
                                    };

                                    let mut packet_vec =
                                        vec![0; 2 + header.encoded_len() + response.len()];
                                    let mut packet = packet_vec.as_mut_slice();
                                    packet
                                        .write_u16::<BigEndian>(header.encoded_len() as u16)
                                        .unwrap();
                                    header.encode(&mut packet).unwrap();
                                    packet.write(&response).unwrap();
                                    client_message_tx.send(packet_vec).unwrap();
                                }
                            }
                        }
                    });

                    tokio::spawn(async move {
                        loop {
                            tokio::select! {
                                message = stream.next() => {
                                    if let tokio_tungstenite::tungstenite::Message::Binary(message) = message.unwrap().unwrap() {
                                        let mut message = message.as_slice();
                                        let header_size = message.read_u16::<BigEndian>().unwrap();
                                        let (header, message) = message.split_at(header_size as usize);
                                        let header = bgs::protocol::Header::decode(header).unwrap();
                                        match header.service_id {
                                            0 => {
                                                client_request_tx.send((header, message.to_vec())).unwrap();
                                            }
                                            254 => {
                                                client_service.handle_response(header.token, message.to_vec());
                                            }
                                            _ => todo!()
                                        }
                                    }
                                }
                                message = client_message_rx.recv() => {
                                    stream.send(tokio_tungstenite::tungstenite::Message::Binary(message.unwrap())).await.unwrap();
                                }
                            }
                        }
                    });

                    tokio::spawn(async move {
                        loop {
                            tokio::select! {
                                message = remote_stream.next() => {
                                    if let tokio_tungstenite::tungstenite::Message::Binary(message) = message.unwrap().unwrap() {
                                        let mut message = message.as_slice();
                                        let header_size = message.read_u16::<BigEndian>().unwrap();
                                        let (header, message) = message.split_at(header_size as usize);
                                        let header = bgs::protocol::Header::decode(header).unwrap();
                                        match header.service_id {
                                            0 => {
                                                server_request_tx.send((header, message.to_vec())).unwrap();
                                            }
                                            254 => {
                                                server_service.handle_response(header.token, message.to_vec());
                                            }
                                            _ => todo!()
                                        }
                                    }
                                }
                                message = server_message_rx.recv() => {
                                    remote_stream.send(tokio_tungstenite::tungstenite::Message::Binary(message.unwrap())).await.unwrap();
                                }
                            }
                        }
                    });
                });
            }
        }
        Arguments::Patch { file, uris } => {
            let public_key =
                RsaPublicKey::from_public_key_pem(include_str!("blizzard_certificate_bundle.pub"))
                    .unwrap();
            let mut public_key_n_and_e = public_key.n().to_bytes_le();
            public_key_n_and_e.append(&mut public_key.e().to_bytes_le());

            let mut file_content = std::fs::read(&file).unwrap();
            if let (
                Some(public_key_n_index),
                Some(certificate_bundle_index),
                Some(certificate_bundle_signature_index),
            ) = (
                kmp::kmp_find(&public_key_n_and_e, &file_content),
                kmp::kmp_find(b"{\"Created\":", &file_content),
                kmp::kmp_find(b"}NGIS", &file_content),
            ) {
                // create new certificate bundle
                let certificate_bundle_signature_index = certificate_bundle_signature_index + 1;
                let certs = uris
                    .into_iter()
                    .map(|uri| CertificateBundlePublicKey {
                        uri: uri.clone(),
                        sha256: sha256(
                            &X509::from_pem(&std::fs::read(format!("certs/{uri}.crt")).unwrap())
                                .unwrap()
                                .public_key()
                                .unwrap()
                                .rsa()
                                .unwrap()
                                .public_key_to_der_pkcs1()
                                .unwrap(),
                        ),
                    })
                    .collect::<Vec<_>>();
                let ca_cert_pem = std::fs::read_to_string("certs/root.crt").unwrap();
                let ca_cert = X509::from_pem(ca_cert_pem.as_bytes()).unwrap();
                let certificate_bundle = format!(
                    "{:1$}",
                    serde_json::to_string(&CertificateBundle {
                        created: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        certificates: certs.clone(),
                        public_keys: certs,
                        signing_certificates: vec![CertificateBundleCertificate {
                            data: ca_cert_pem.replace('\n', ""),
                        }],
                        root_ca_public_keys: vec![hex::encode(sha256(
                            &ca_cert
                                .public_key()
                                .unwrap()
                                .rsa()
                                .unwrap()
                                .public_key_to_der_pkcs1()
                                .unwrap()
                        ))],
                    })
                    .unwrap(),
                    certificate_bundle_signature_index - certificate_bundle_index
                );

                // create new private key and sign certificate bundle
                let mut rng = rand::thread_rng();
                let private_key = RsaPrivateKey::new(&mut rng, public_key.size() * 8).unwrap();
                let private_key_n = private_key.n().to_bytes_le();
                let private_key_e = private_key.e().to_bytes_le();
                let signing_key = SigningKey::<Sha256>::new_with_prefix(private_key);
                let signature = signing_key
                    .sign_with_rng(
                        &mut rng,
                        format!("{}Blizzard Certificate Bundle", certificate_bundle).as_bytes(),
                    )
                    .to_vec();

                // update public key, certificate bundle and signature
                let public_key_e_index = public_key_n_index + private_key_n.len();
                file_content.splice(public_key_n_index..public_key_e_index, private_key_n);
                file_content.splice(
                    public_key_e_index..public_key_e_index + private_key_e.len(),
                    private_key_e,
                );
                file_content.splice(
                    certificate_bundle_index..certificate_bundle_signature_index,
                    certificate_bundle.into_bytes(),
                );
                file_content.splice(
                    certificate_bundle_signature_index + 4
                        ..certificate_bundle_signature_index + 4 + signature.len(),
                    signature.into_iter().rev(),
                );
                std::fs::write(file, file_content).unwrap();
            } else {
                println!("Public key, certificate bundle or signature not found")
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CertificateBundle {
    #[serde(rename = "Created")]
    created: u64,
    #[serde(rename = "Certificates")]
    certificates: Vec<CertificateBundlePublicKey>,
    #[serde(rename = "PublicKeys")]
    public_keys: Vec<CertificateBundlePublicKey>,
    #[serde(rename = "SigningCertificates")]
    signing_certificates: Vec<CertificateBundleCertificate>,
    #[serde(rename = "RootCAPublicKeys")]
    root_ca_public_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CertificateBundlePublicKey {
    #[serde(rename = "Uri")]
    uri: String,
    #[serde(rename = "ShaHashPublicKeyInfo", with = "hex")]
    sha256: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct CertificateBundleCertificate {
    #[serde(rename = "RawData")]
    data: String,
}
