use serde::{Deserialize, Serialize};

// it's not the awful kind I promise
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum OOBEStep {
    SetupPath,
    Resume, // jump to a step
    Welcome,
    LicenseAgreement,
    SetupCerts,
    StartProxy
}