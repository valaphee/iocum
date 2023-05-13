use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use clap::{arg, Parser};
use hyper::{
    body::Incoming,
    client,
    header::{HeaderValue, HOST},
    http::uri::Authority,
    server,
    service::service_fn,
    Error, Request, Uri,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerConfig, ServerName},
    TlsAcceptor, TlsConnector,
};

use staxtls::ResolvesServerCertAutogen;

#[derive(Parser)]
#[command(about)]
struct Arguments {
    /// Uri to bind to
    #[arg(long)]
    uri: Uri,
    /// Uri to connect to, if empty transparent mode will be used
    #[arg(long)]
    remote_uri: Uri,
    /// Default server name indicator
    #[arg(long)]
    default_sni: Option<String>,
    /// Use HTTP 2.0
    #[arg(long)]
    http2: bool,
}

#[tokio::main]
async fn main() {
    // parse arguments
    let Arguments {
        uri,
        remote_uri,
        default_sni,
        http2,
    } = Arguments::parse();
    // bind listener
    let listener = TcpListener::bind(format!(
        "{}:{}",
        uri.host().unwrap(),
        uri.port().map_or(443, |port| port.as_u16())
    ))
    .await
    .unwrap();
    // setup tls acceptor
    let mut tls_server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(ResolvesServerCertAutogen::new(
            "certs",
            default_sni.unwrap_or(uri.host().unwrap().to_string()),
        )));
    if http2 {
        tls_server_config.alpn_protocols = vec![b"h2".to_vec()];
    }
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_config));
    // setup tls connector
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
    if http2 {
        tls_client_config.alpn_protocols = vec![b"h2".to_vec()];
    }
    let tls_connector = TlsConnector::from(Arc::new(tls_client_config));
    // parse remote address
    let remote_addr = format!(
        "{}:{}",
        remote_uri.host().unwrap(),
        remote_uri.port().map_or(443, |port| port.as_u16())
    )
    .to_socket_addrs()
    .unwrap()
    .next()
    .unwrap();
    // for each socket
    loop {
        let (socket, address) = listener.accept().await.unwrap();
        // spawn new task to handle socket
        let tls_acceptor = tls_acceptor.clone();
        let tls_connector = tls_connector.clone();
        let remote_host = remote_uri.host().unwrap().to_string();
        tokio::task::spawn(async move {
            handle(
                socket,
                address,
                tls_acceptor,
                tls_connector,
                remote_host,
                remote_addr,
                http2,
            )
        });
    }
}

async fn handle(
    socket: TcpStream,
    address: SocketAddr,
    tls_acceptor: TlsAcceptor,
    tls_connector: TlsConnector,
    remote_host: String,
    remote_addr: SocketAddr,
    http2: bool,
) {
    // handle tls and wrap socket
    let socket = tls_acceptor.accept(socket).await.unwrap();
    let service = service_fn(move |mut request: Request<Incoming>| {
        // log request
        println!("<< {address}");
        println!(
            "{} {} {:?}",
            request.method(),
            request.uri(),
            request.version()
        );
        for (header_name, header_value) in request.headers().iter() {
            println!("{}: {}", header_name, header_value.to_str().unwrap());
        }
        // transform request
        let request = if http2 {
            let (mut request_parts, request_body) = request.into_parts();
            let mut uri_parts = request_parts.uri.into_parts();
            uri_parts.authority = Some(Authority::try_from(remote_host.clone()).unwrap());
            request_parts.uri = Uri::from_parts(uri_parts).unwrap();
            Request::from_parts(request_parts, request_body)
        } else {
            request
                .headers_mut()
                .insert(HOST, HeaderValue::from_str(&remote_host).unwrap());
            request
        };
        // service function
        let tls_connector = tls_connector.clone();
        let remote_host = remote_host.clone();
        async move {
            // connect
            let remote_socket = TcpStream::connect(&remote_addr).await.unwrap();
            // handle tls and wrap socket
            let remote_socket = tls_connector
                .connect(
                    ServerName::try_from(remote_host.as_str()).unwrap(),
                    remote_socket,
                )
                .await
                .unwrap();
            // request
            let response = if http2 {
                let (mut sender, connection) =
                    client::conn::http2::handshake(TokioExecutor, remote_socket)
                        .await
                        .unwrap();
                tokio::task::spawn(async move {
                    connection.await.unwrap();
                });
                sender.send_request(request).await.unwrap()
            } else {
                let (mut sender, connection) =
                    client::conn::http1::handshake(remote_socket).await.unwrap();
                tokio::task::spawn(async move {
                    connection.await.unwrap();
                });
                sender.send_request(request).await.unwrap()
            };
            // log response
            println!(">> {address}");
            println!("{:?} {}", response.version(), response.status().as_str());
            for (header_name, header_value) in response.headers().iter() {
                println!("{}: {}", header_name, header_value.to_str().unwrap());
            }
            Ok::<_, Error>(response)
        }
    });
    // handle http
    if http2 {
        server::conn::http2::Builder::new(TokioExecutor)
            .serve_connection(socket, service)
            .await
            .unwrap();
    } else {
        server::conn::http1::Builder::new()
            .serve_connection(socket, service)
            .await
            .unwrap();
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
