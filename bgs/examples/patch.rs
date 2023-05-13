use std::path::PathBuf;

use clap::Parser;
use openssl::{sha::sha256, x509::X509};
use rsa::{
    pkcs1v15::SigningKey,
    pkcs8::DecodePublicKey,
    signature::{RandomizedSigner, SignatureEncoding},
    PublicKeyParts, RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Parser)]
struct Arguments {
    file: PathBuf,
    #[arg(required(true))]
    uris: Vec<String>,
}

fn main() {
    // parse arguments
    let Arguments { file, uris } = Arguments::parse();
    // import public key to be replaced
    let public_key =
        RsaPublicKey::from_public_key_pem(include_str!("blizzard_certificate_bundle.pub")).unwrap();
    let mut public_key_n_and_e = public_key.n().to_bytes_le();
    public_key_n_and_e.append(&mut public_key.e().to_bytes_le());
    // search for public key
    let mut file_content = std::fs::read(&file).unwrap();
    let Some(public_key_n_index) = kmp::kmp_find(&public_key_n_and_e, &file_content) else {
        eprintln!("public key not found");
        return;
    };
    // search for certificate bundle
    let Some(certificate_bundle_index) = kmp::kmp_find(b"{\"Created\":", &file_content) else {
        eprintln!("certificate bundle not found");
        return;
    };
    // search for certificate bundle signature
    let Some(certificate_bundle_signature_index) = kmp::kmp_find(b"}NGIS", &file_content) else {
        eprintln!("certificate bundle signature not found");
        return;
    };
    // create certificate bundle
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
            created: 1612222344,
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
    // create private key
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, public_key.size() * 8).unwrap();
    let private_key_n = private_key.n().to_bytes_le();
    let private_key_e = private_key.e().to_bytes_le();
    let signing_key = SigningKey::<Sha256>::new_with_prefix(private_key);
    // sign certificate bundle
    let signature = signing_key
        .sign_with_rng(
            &mut rng,
            format!("{}Blizzard Certificate Bundle", certificate_bundle).as_bytes(),
        )
        .to_vec();
    // replace public key, certificate bundle and signature
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
