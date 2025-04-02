#[derive(Debug, Clone)]
pub struct DomainResult {
    pub name: String,
    pub available: bool,
    pub premium: bool,
}

impl DomainResult {
    pub fn new(name: String, available: bool) -> Self {
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
