use std::path::PathBuf;

use log::info;
use rcgen::{generate_simple_self_signed, BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyUsagePurpose};

use crate::{config::Config, resource::{FileResource, Resource}};

pub trait CertDerivable {
    fn derive_cert(&mut self) -> Result<(PathBuf, PathBuf), std::io::Error>;
}

impl CertDerivable for Config {
    fn derive_cert(&mut self) -> Result<(PathBuf, PathBuf), std::io::Error> {
        
        let cert_path = self.data_dir.join("cert.pem");
        let key_path = self.data_dir.join("key.pem");

        let mut params = CertificateParams::default();
        let mut distinguished_name = DistinguishedName::new();
    
        distinguished_name.push(DnType::CommonName, "Telescope MITM Proxy");
        distinguished_name.push(DnType::OrganizationName, "The quieter you are, the more you hear"); // stolen from some kali linux thing
        distinguished_name.push(DnType::CountryName, "US");
        distinguished_name.push(DnType::StateOrProvinceName, "NA");
        distinguished_name.push(DnType::LocalityName, "NA");
    
        params.distinguished_name = distinguished_name;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];
    
        let key_pair = rcgen::KeyPair::generate().expect("keypair generation failed");
        let cert = params.self_signed(&key_pair).expect("self-signed cert generation failed");
        let private_key = key_pair.serialize_pem();

        // write cert and key to disk
        std::fs::write(&cert_path, cert.pem())?;
        std::fs::write(&key_path, private_key)?;

        info!("wrote certs to {} and {}", cert_path.display(), key_path.display());

        self.ca.certificate = Resource::File(FileResource::new(cert_path.to_str().unwrap()));
        self.ca.key_pair = Resource::File(FileResource::new(key_path.to_str().unwrap()));

        Ok(
            (cert_path, key_path)
        )
    }
}