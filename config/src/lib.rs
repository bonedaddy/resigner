#![allow(clippy::needless_lifetimes)]
#![allow(clippy::bool_assert_comparison)]
#![allow(non_upper_case_globals)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use simplelog::*;
use solana_clap_utils::keypair::signer_from_path;
use solana_remote_wallet::remote_wallet;

use solana_sdk::signature::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;

use std::fs;

/// main configuration object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub key_path: String,
    pub rpc_endpoints: RPCs,
}

impl Configuration {
    /// saves the configuration file to disk.
    /// when `store_market_data`is false, the `Markets.data` field is reset to default
    pub fn save(&self, path: &str) -> Result<()> {
        let data = serde_yaml::to_string(self)?;
        fs::write(path, data).expect("failed to write to file");
        Ok(())
    }
    pub fn load(path: &str, init_log: bool) -> Result<Configuration> {
        let data = fs::read(path).expect("failed to read file");
        let config: Configuration = serde_yaml::from_slice(data.as_slice())?;
        if init_log {
            config.init_log(false)?;
        }
        Ok(config)
    }
    pub fn payer(&self) -> Keypair {
        read_keypair_file(self.key_path.clone()).expect("failed to read keypair file")
    }
    /// like `payer` but returns a signer instead and can be used with key files or hardware wallets
    pub fn payer_signer(&self, matches: Option<&clap::ArgMatches>) -> Result<Box<dyn Signer>> {
        if self.key_path.starts_with("usb://ledger") {
            self.usb_payer2(matches.unwrap())
        } else {
            Ok(Box::new(self.payer()))
        }
    }
    pub fn usb_payer2(&self, matches: &clap::ArgMatches) -> Result<Box<dyn Signer>> {
        let mut wallet_manager = remote_wallet::maybe_wallet_manager().unwrap();
        match signer_from_path(matches, &self.key_path, &self.key_path, &mut wallet_manager) {
            Err(err) => Err(anyhow!(
                "encountered error retrieving signer from path {:#?}",
                err
            )),
            Ok(signer) => Ok(signer),
        }
    }
    /// if file_log is true, log to both file and stdout
    /// otherwise just log to stdout
    pub fn init_log(&self, _file_log: bool) -> Result<()> {
        TermLogger::init(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_location_level(LevelFilter::Error)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )?;
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RPCs {
    pub primary_endpoint: RPCEndpoint,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RPCEndpoint {
    pub http_url: String,
    pub ws_url: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            rpc_endpoints: RPCs {
                primary_endpoint: RPCEndpoint {
                    http_url: "https://....quiknode.pro/.../".to_string(),
                    ws_url: "ws://api.mainnet-beta.solana.com".to_string(),
                },
            },
            key_path: "".to_string(),
        }
    }
}
