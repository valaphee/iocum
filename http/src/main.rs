extern crate core;

use std::{net::ToSocketAddrs, sync::Arc};
use std::collections::{hash_map, HashMap};
use std::fs::File;
use std::io::Read;

use clap::{arg, Parser};
use flate2::read::GzDecoder;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, client, http::uri::Authority, server, service::service_fn, Error, Request, Uri, Response};
use hyper::body::Bytes;
use hyper::header::{HeaderValue, SET_COOKIE};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerConfig, ServerName},
    TlsAcceptor, TlsConnector,
};

use staxtls::ResolvesServerCertAutogen;

#[derive(Parser)]
enum Arguments {
    Mitm {
        remote_addr: Uri,
        #[arg(long)]
        local_addr: Option<Uri>,
        #[arg(long)]
        default_sni: Option<String>,
        #[arg(long)]
        dump: bool
    },
}

#[tokio::main]
async fn main() {
    let Arguments::Mitm {
        remote_addr,
        local_addr,
        default_sni,
        dump: _,
    } = Arguments::parse();

    let local_addr = local_addr.unwrap();
    let listener = TcpListener::bind(format!(
        "{}:{}",
        local_addr.host().unwrap(),
        local_addr.port().map_or(443, |port| port.as_u16())
    ))
    .await
    .unwrap();

    // setup tls server config
    let mut tls_server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(ResolvesServerCertAutogen::new(
            "certs",
            default_sni.unwrap_or(local_addr.host().unwrap().to_string()),
        )));
    tls_server_config.alpn_protocols = vec![b"h2".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_config));

    // setup tls client config
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
        |trust_anchor| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                trust_anchor.subject,
                trust_anchor.spki,
                trust_anchor.name_constraints,
            )
        },
    ));
    let mut tls_client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    tls_client_config.alpn_protocols = vec![b"h2".to_vec()];
    let tls_connector = TlsConnector::from(Arc::new(tls_client_config));

    let remote_host = remote_addr.host().unwrap().to_string();
    let remote_addr = format!(
        "{}:{}",
        remote_addr.host().unwrap(),
        remote_addr.port().map_or(443, |port| port.as_u16())
    )
    .to_socket_addrs()
    .unwrap()
    .next()
    .unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let tls_acceptor = tls_acceptor.clone();
        let tls_connector = tls_connector.clone();
        let remote_host = remote_host.clone();

        tokio::task::spawn(async move {
            let stream = tls_acceptor.accept(stream).await.unwrap();
            let service = service_fn(move |request: Request<Incoming>| {
                println!("-----BEGIN HTTP REQUEST-----");
                println!(
                    "{} {} {:?}",
                    request.method(),
                    request.uri(),
                    request.version()
                );
                for (header_name, header_value) in request.headers().iter() {
                    println!("{}: {}", header_name, header_value.to_str().unwrap());
                }
                println!("-----END HTTP REQUEST-----");

                let tls_connector = tls_connector.clone();
                let remote_host = remote_host.clone();

                let request = {
                    // rewrite uri authority
                    let (mut request_parts, request_body) = request.into_parts();
                    let mut uri_parts = request_parts.uri.into_parts();
                    uri_parts.authority = Some(Authority::try_from(remote_host.clone()).unwrap());
                    request_parts.uri = Uri::from_parts(uri_parts).unwrap();
                    Request::from_parts(request_parts, request_body)
                };

                async move {
                    let stream = TcpStream::connect(&remote_addr).await.unwrap();
                    let stream = tls_connector
                        .connect(ServerName::try_from(remote_host.as_str()).unwrap(), stream)
                        .await
                        .unwrap();
                    let (mut sender, connection) =
                        client::conn::http2::handshake(TokioExecutor, stream)
                            .await
                            .unwrap();
                    tokio::task::spawn(async move {
                        connection.await.unwrap();
                    });

                    let mut response = sender.send_request(request).await.unwrap();
                    println!("-----BEGIN HTTP RESPONSE-----");
                    println!("{:?} {}", response.version(), response.status().as_str());
                    for (header_name, header_value) in response.headers().iter() {
                        println!("{}: {}", header_name, header_value.to_str().unwrap());
                    }
                    println!("-----END HTTP RESPONSE-----");

                    // remove set-cookie header domain
                    if let Some(mut cookie_header) = response.headers_mut().get_mut(SET_COOKIE) {
                        let value = cookie_header.to_str().unwrap();
                        *cookie_header = HeaderValue::from_str(&value.split(';').filter_map(|key_value_pair| {
                            let mut key_value_split = key_value_pair.splitn(2, '=');
                            let key = key_value_split.next().unwrap().trim();
                            if key == "domain" {
                                return None;
                            }
                            Some(if let Some(value) = key_value_split.next() {
                                format!("{key}={value}")
                            } else {
                                format!("{key}")
                            })
                        }).collect::<Vec<_>>().join(";")).unwrap();
                    }

                    Ok::<_, Error>(response)
                }
            });
            server::conn::http2::Builder::new(TokioExecutor)
                .serve_connection(stream, service)
                .await
                .unwrap();
        });
    }
}

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}
