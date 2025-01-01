use std::path::PathBuf;

use rcgen::{generate_simple_self_signed, BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyUsagePurpose};

use crate::config::Config;

pub trait CertDerivable {
    fn derive_cert(&self) -> Result<(PathBuf, PathBuf), std::io::Error>;
}

impl CertDerivable for Config {
    fn derive_cert(&self) -> Result<(PathBuf, PathBuf), std::io::Error> {
        
        let cert_path = self.data_dir.join("cert.pem");
        let key_path = self.data_dir.join("key.pem");

        let mut params = CertificateParams::default();
        let mut distinguished_name = DistinguishedName::new();
    
        distinguished_name.push(DnType::CommonName, "Hudsucker Industries");
        distinguished_name.push(DnType::OrganizationName, "Hudsucker Industries");
        distinguished_name.push(DnType::CountryName, "US");
        distinguished_name.push(DnType::StateOrProvinceName, "NY");
        distinguished_name.push(DnType::LocalityName, "NYC");
    
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

        Ok(
            (cert_path, key_path)
        )
    }
}