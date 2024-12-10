use crate::store::{Storage, StorageKind};
use crate::{account::wallet::Wallet, account::Account};
use clap::{Arg, Command};
use clap_derive::Subcommand; // Import from clap_derive
use serde::{Deserialize, Serialize};

struct
