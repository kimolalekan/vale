use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Processing => "Processing",
            TransactionStatus::Completed => "Completed",
            TransactionStatus::Failed => "Failed",
            TransactionStatus::Cancelled => "Cancelled",
        }
    }
}
