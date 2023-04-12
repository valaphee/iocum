use std::sync::Arc;

use clap::{arg, Parser};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, handshake::server};
use url::Url;

use tlsbogie::ResolvesServerCertAutogen;

#[derive(Parser)]
struct Arguments {
    #[arg(long)]
    local_addr: Option<Url>,
    #[arg(long)]
    remote_addr: Url,
    #[arg(long)]
    default_sni: Option<String>,
}

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    let local_addr = arguments.local_addr.unwrap();
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
    let mut tls_server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(ResolvesServerCertAutogen::new(
            "certs",
            arguments
                .default_sni
                .unwrap_or(local_addr.host().unwrap().to_string()),
        )));
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_config));

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let tls_acceptor = tls_acceptor.clone();
        let remote_addr = arguments.remote_addr.clone();

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
