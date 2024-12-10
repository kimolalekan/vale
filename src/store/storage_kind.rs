#[derive(Debug, Clone, Copy)]
pub enum StorageKind {
    Account,
    Transaction,
    Contract,
    Chain,
    Analytics,
    Index,
}

impl StorageKind {
    pub fn name(&self) -> &str {
        match self {
            StorageKind::Account => "accounts",
            StorageKind::Transaction => "transactions",
            StorageKind::Contract => "contracts",
            StorageKind::Chain => "blockchains",
            StorageKind::Analytics => "analytics",
            StorageKind::Index => "index",
        }
    }
}
