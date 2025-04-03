use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResult {
    pub name: String,
    pub available: bool,
    pub premium: bool,
}

impl DomainResult {
    pub fn _new(name: String, available: bool) -> Self {
        Self {
            name,
            available,
            premium: false,
        }
    }

    pub fn new_with_premium(name: String, available: bool, premium: bool) -> Self {
        Self {
            name,
            available,
            premium,
        }
    }
}
