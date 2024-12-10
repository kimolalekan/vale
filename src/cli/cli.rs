use crate::store::{Storage, StorageKind};
use crate::{account::wallet::Wallet, account::Account};
use clap::{Arg, Command};
use clap_derive::Subcommand; // Import from clap_derive
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balance {
    pub address: String,
    pub balance: f64,
}

#[derive(Subcommand, Debug)] // Derive Subcommand
pub enum CliCommand {
    CreateAccount,
    GetBalance {
        address: String,
        private_key: String,
    },
    GetAccount {
        private_key: String,
    },
}

pub fn cli() {
    let matches = Command::new("Crypto CLI")
        .version("1.0")
        .author("Your Name <youremail@example.com>")
        .about("CLI for managing cryptocurrency accounts")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("create-account").about("Creates a new account"))
        .subcommand(
            Command::new("get-balance")
                .about("Gets the balance of an account")
                .arg(
                    Arg::new("address")
                        .help("Address of the account to check")
                        .required(true) // Marks the argument as required
                        .value_name("ADDRESS"), // Sets the name for the argument
                )
                .arg(
                    Arg::new("private_key")
                        .help("Private key of the account to check balance")
                        .required(true)
                        .value_name("PRIVATE_KEY"),
                ),
        )
        .subcommand(
            Command::new("get-account")
                .about("Gets account details by private key")
                .arg(
                    Arg::new("private_key")
                        .help("Private key of the account")
                        .required(true)
                        .value_name("PRIVATE_KEY"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("create-account", _)) => match Account::new() {
            Ok(account_with_private_key) => {
                println!(
                    "Account created successfully: {:?}",
                    account_with_private_key
                );
            }
            Err(e) => {
                eprintln!("Error creating account: {}", e);
            }
        },
        Some(("get-balance", sub_m)) => {
            let address = sub_m.get_one::<String>("address").unwrap();
            let private_key = sub_m.get_one::<String>("private_key").unwrap();

            match Account::get_balance(address.to_string(), private_key.to_string()) {
                Ok(balance) => {
                    println!("Balance for {}: {}", balance.address, balance.balance);
                }
                Err(e) => {
                    eprintln!("Error retrieving balance: {}", e);
                }
            }
        }
        Some(("get-account", sub_m)) => {
            let private_key = sub_m.get_one::<String>("private_key").unwrap();

            match Account::get_account(private_key.to_string()) {
                Ok(account) => {
                    println!("Account details: {:?}", account);
                }
                Err(e) => {
                    eprintln!("Error retrieving account: {}", e);
                }
            }
        }
        _ => {
            eprintln!("Invalid command.");
        }
    }
}
