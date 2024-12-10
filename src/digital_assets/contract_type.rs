#[derive(Debug, Clone, Copy)]
pub enum ContractType {
    Identity,
    Vote,
    Document,
    Text,
}

impl ContractType {
    pub fn name(&self) -> &str {
        match self {
            ContractType::Identity => "aggreement",
            ContractType::Vote => "vote",
            ContractType::Document => "document",
            ContractType::Text => "text",
        }
    }
}
