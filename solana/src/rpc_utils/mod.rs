//! provides rpc utils for the solana network
use anyhow::Result;

use serde_json::json;

use config::RPCs as ConfigRPCs;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest, rpc_response::RpcBlockhash};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct RPCs {
    pub primary_endpoint: RPCEndpoint,
}

#[derive(Clone)]
pub struct RPCEndpoint {
    pub http_url: String,
    pub ws_url: String,
}

impl From<&ConfigRPCs> for RPCs {
    fn from(rpcs: &ConfigRPCs) -> Self {
        Self {
            primary_endpoint: RPCEndpoint {
                http_url: rpcs.primary_endpoint.http_url.clone(),
                ws_url: rpcs.primary_endpoint.ws_url.clone(),
            },
        }
    }
}

impl RPCs {
    // returns the primary rpc provider
    pub fn get_rpc_client(&self, ws: bool, commitment: Option<CommitmentConfig>) -> RpcClient {
        if !ws {
            match commitment {
                Some(commitment) => {
                    return RpcClient::new_with_commitment(
                        self.primary_endpoint.http_url.clone(),
                        commitment,
                    );
                }
                None => {
                    return RpcClient::new_with_commitment(
                        self.primary_endpoint.http_url.clone(),
                        CommitmentConfig::confirmed(),
                    );
                }
            }
        }
        match commitment {
            Some(commitment) => {
                RpcClient::new_with_commitment(self.primary_endpoint.ws_url.clone(), commitment)
            }
            None => RpcClient::new_with_commitment(
                self.primary_endpoint.ws_url.clone(),
                CommitmentConfig::confirmed(),
            ),
        }
    }
}
pub fn get_blockhash_fast(rpc: &Arc<RpcClient>, commitment: CommitmentConfig) -> Result<Hash> {
    let RpcBlockhash {
        blockhash,
        last_valid_block_height: _,
    } = rpc
        .send::<solana_client::rpc_response::Response<RpcBlockhash>>(
            RpcRequest::GetLatestBlockhash,
            json!([commitment]),
        )?
        .value;
    Ok(Hash::from_str(&blockhash)?)
}
