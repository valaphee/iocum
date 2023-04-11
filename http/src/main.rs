use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    convert::Infallible,
    net::ToSocketAddrs,
    sync::Arc,
};

use clap::Parser;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http2, service::service_fn, Request, Response};
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{
        extension::{
            AuthorityKeyIdentifier, BasicConstraints, SubjectAlternativeName, SubjectKeyIdentifier,
        },
        X509Builder, X509NameBuilder, X509,
    },
};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls,
    rustls::{
        server::{ClientHello, ResolvesServerCert},
        sign::CertifiedKey,
        Certificate, PrivateKey, ServerConfig,
    },
    TlsAcceptor,
};

#[derive(Parser)]
enum Arguments {
    Client { address: String },
    Server { address: String },
}

#[tokio::main]
async fn main() {
    match Arguments::parse() {
        Arguments::Client { .. } => {}
        Arguments::Server { address } => {
            let address = address.to_socket_addrs().unwrap().next().unwrap();
            let listener = TcpListener::bind(address).await.unwrap();

            let mut tls_config = ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(ResolvesServerCertAutogen::new()));
            tls_config.alpn_protocols = vec![b"h2".to_vec()];
            let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let tls_acceptor = tls_acceptor.clone();

                tokio::task::spawn(async move {
                    let stream = tls_acceptor.accept(stream).await.unwrap();
                    http2::Builder::new(TokioExecutor)
                        .serve_connection(stream, service_fn(log))
                        .await
                        .unwrap();
                });
            }
        }
    }
}

async fn log(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

struct ResolvesServerCertAutogen {
    ca_key_pair: PKey<Private>,
    ca_cert: X509,

    issued_certs: RefCell<HashMap<String, Arc<CertifiedKey>>>,
}

unsafe impl Sync for ResolvesServerCertAutogen {}

impl ResolvesServerCertAutogen {
    fn new() -> Self {
        let ca_key_pair = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();

        let mut ca_cert_name = X509NameBuilder::new().unwrap();
        ca_cert_name
            .append_entry_by_text("CN", "malebolge")
            .unwrap();
        let ca_cert_name = ca_cert_name.build();

        let mut ca_cert = X509Builder::new().unwrap();
        ca_cert
            .set_not_before(&Asn1Time::days_from_now(0).unwrap())
            .unwrap();
        ca_cert
            .set_not_after(&Asn1Time::days_from_now(365).unwrap())
            .unwrap();
        ca_cert.set_version(2).unwrap();
        ca_cert
            .set_serial_number({
                let mut serial_number = BigNum::new().unwrap();
                serial_number
                    .rand(159, MsbOption::MAYBE_ZERO, false)
                    .unwrap();
                &serial_number.to_asn1_integer().unwrap()
            })
            .unwrap();
        ca_cert.set_issuer_name(&ca_cert_name).unwrap();
        ca_cert.set_subject_name(&ca_cert_name).unwrap();
        ca_cert.set_pubkey(&ca_key_pair).unwrap();
        ca_cert
            .append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        ca_cert
            .append_extension(
                SubjectKeyIdentifier::new()
                    .build(&ca_cert.x509v3_context(None, None))
                    .unwrap(),
            )
            .unwrap();
        ca_cert.sign(&ca_key_pair, MessageDigest::sha256()).unwrap();

        Self {
            ca_key_pair,
            ca_cert: ca_cert.build(),
            issued_certs: RefCell::new(Default::default()),
        }
    }
}

impl ResolvesServerCert for ResolvesServerCertAutogen {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        if let Some(name) = client_hello.server_name() {
            Some(
                match self.issued_certs.borrow_mut().entry(name.to_string()) {
                    Entry::Occupied(entry) => entry.get().clone(),
                    Entry::Vacant(entry) => {
                        let key_pair = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();

                        let mut cert_name = X509NameBuilder::new().unwrap();
                        cert_name.append_entry_by_text("CN", name).unwrap();
                        let cert_name = cert_name.build();

                        let mut cert = X509Builder::new().unwrap();
                        cert.set_not_before(&Asn1Time::days_from_now(0).unwrap())
                            .unwrap();
                        cert.set_not_after(&Asn1Time::days_from_now(365).unwrap())
                            .unwrap();
                        cert.set_version(2).unwrap();
                        cert.set_serial_number({
                            let mut serial_number = BigNum::new().unwrap();
                            serial_number
                                .rand(159, MsbOption::MAYBE_ZERO, false)
                                .unwrap();
                            &serial_number.to_asn1_integer().unwrap()
                        })
                        .unwrap();
                        cert.set_issuer_name(self.ca_cert.subject_name()).unwrap();
                        cert.set_subject_name(&cert_name).unwrap();
                        cert.set_pubkey(&key_pair).unwrap();
                        cert.append_extension(BasicConstraints::new().build().unwrap())
                            .unwrap();
                        cert.append_extension(
                            SubjectKeyIdentifier::new()
                                .build(&cert.x509v3_context(Some(&self.ca_cert), None))
                                .unwrap(),
                        )
                        .unwrap();
                        cert.append_extension(
                            AuthorityKeyIdentifier::new()
                                .keyid(false)
                                .issuer(false)
                                .build(&cert.x509v3_context(Some(&self.ca_cert), None))
                                .unwrap(),
                        )
                        .unwrap();
                        cert.append_extension(
                            SubjectAlternativeName::new()
                                .dns(name)
                                .build(&cert.x509v3_context(Some(&self.ca_cert), None))
                                .unwrap(),
                        )
                        .unwrap();
                        cert.sign(&self.ca_key_pair, MessageDigest::sha256())
                            .unwrap();
                        let cert = cert.build();

                        entry
                            .insert(
                                CertifiedKey::new(
                                    vec![
                                        Certificate(cert.to_der().unwrap()),
                                        Certificate(self.ca_cert.to_der().unwrap()),
                                    ],
                                    rustls::sign::any_supported_type(&PrivateKey(
                                        key_pair.private_key_to_der().unwrap(),
                                    ))
                                    .unwrap(),
                                )
                                .into(),
                            )
                            .clone()
                    }
                },
            )
        } else {
            None
        }
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
