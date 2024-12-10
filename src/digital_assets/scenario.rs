use super::contract_type::ContractType;

pub struct Scenario {
    pub title: String,
    pub contract_type: ContractType,
    pub title: String<60>,
    pub limit: String,
    pub owner: String,
    pub hash: String,
}

impl Scenario {
    pub fn new() -> Self {}
}
