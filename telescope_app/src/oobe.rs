use serde::{Deserialize, Serialize};

// it's not the awful kind I promise
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum OOBEStep {
    Resume, // jump to a step
    Welcome,
    LicenseAgreement,
    SetupPath,
    SetupCerts,
    StartProxy
}