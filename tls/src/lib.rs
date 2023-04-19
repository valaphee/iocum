use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{
        extension::{
            AuthorityKeyIdentifier, BasicConstraints, SubjectAlternativeName, SubjectKeyIdentifier,
        },
        X509Builder, X509NameBuilder, X509,
    },
};
use rustls::{
    server::{ClientHello, ResolvesServerCert},
    sign::CertifiedKey,
    Certificate, PrivateKey,
};

pub struct ResolvesServerCertAutogen {
    path: PathBuf,
    default_sni: String,

    ca_key_pair: PKey<Private>,
    ca_cert: X509,

    certs: RefCell<HashMap<String, Arc<CertifiedKey>>>,
}

unsafe impl Sync for ResolvesServerCertAutogen {}

impl ResolvesServerCertAutogen {
    pub fn new<P: AsRef<Path>>(path: P, default_sni: String) -> Self {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(path.clone()).unwrap();

        let (ca_key_pair, ca_cert) = if let (Ok(ca_cert_pem), Ok(ca_private_key_pem)) = (
            std::fs::read(path.join("root.crt")),
            std::fs::read(path.join("root.pvk")),
        ) {
            (
                PKey::private_key_from_pem(&ca_private_key_pem).unwrap(),
                X509::from_pem(&ca_cert_pem).unwrap(),
            )
        } else {
            let ca_key_pair = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();

            let mut ca_cert_name = X509NameBuilder::new().unwrap();
            ca_cert_name
                .append_entry_by_nid(Nid::COMMONNAME, "Root CA")
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
                        .rand(128, MsbOption::MAYBE_ZERO, false)
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
            let ca_cert = ca_cert.build();

            std::fs::write(path.join("root.crt"), ca_cert.to_pem().unwrap()).unwrap();
            std::fs::write(
                path.join("root.pvk"),
                ca_key_pair.private_key_to_pem_pkcs8().unwrap(),
            )
            .unwrap();

            (ca_key_pair, ca_cert)
        };

        Self {
            path,
            default_sni,
            ca_key_pair,
            ca_cert,
            certs: RefCell::default(),
        }
    }
}

impl ResolvesServerCert for ResolvesServerCertAutogen {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let sni = client_hello.server_name().unwrap_or(&self.default_sni);
        Some(match self.certs.borrow_mut().entry(sni.to_string()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let (key_pair, cert) = if let (Ok(cert_pem), Ok(private_key_pem)) = (
                    std::fs::read(self.path.join(format!("{}.crt", sni))),
                    std::fs::read(self.path.join(format!("{}.pvk", sni))),
                ) {
                    (
                        PKey::private_key_from_pem(&private_key_pem).unwrap(),
                        X509::from_pem(&cert_pem).unwrap(),
                    )
                } else {
                    let key_pair = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();

                    let mut cert_name = X509NameBuilder::new().unwrap();
                    cert_name.append_entry_by_nid(Nid::COMMONNAME, sni).unwrap();
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
                            .rand(128, MsbOption::MAYBE_ZERO, false)
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
                            .dns(sni)
                            .build(&cert.x509v3_context(Some(&self.ca_cert), None))
                            .unwrap(),
                    )
                    .unwrap();
                    cert.sign(&self.ca_key_pair, MessageDigest::sha256())
                        .unwrap();
                    let cert = cert.build();

                    std::fs::write(
                        self.path.join(format!("{}.crt", sni)),
                        cert.to_pem().unwrap(),
                    )
                    .unwrap();
                    std::fs::write(
                        self.path.join(format!("{}.pvk", sni)),
                        key_pair.private_key_to_pem_pkcs8().unwrap(),
                    )
                    .unwrap();

                    (key_pair, cert)
                };

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
        })
    }
}
