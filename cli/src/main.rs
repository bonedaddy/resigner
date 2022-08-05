use ::config::Configuration;
use anyhow::{anyhow, Result};
use clap::{App, Arg, SubCommand};
use helpers::get_config;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    message::SanitizedMessage,
};
use solana_sdk::{signer::Signer, transaction::Transaction};

mod config;
mod helpers;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let matches = App::new("resigner")
        .version("0.0.1")
        .author("bonedaddy")
        .about("transaction resigner and broadcaster")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("sets the config file")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("configuration management commands")
                .subcommands(vec![

                    SubCommand::with_name("new")
                        .about("generates a new and empty configuration file"),
                ]),
        )
        .subcommand(
            SubCommand::with_name("resign")
            .about("resign and resend base64 encoded transaction payload")
            .long_about("by default resends entire transaction, but can set such that if a txn has multiple instructions, each instruction is sent in a separate transaction")
            .arg(
                Arg::with_name("input-file")
                .long("input-file")
                .help("file to read encoded txn from")
                .takes_value(true)
                .required(true)
                .value_name("FILE")
            )
            .arg(
                Arg::with_name("one-by-one")
                .long("one-by-one")
                .help("instead of decoding txn and sending, decode txn, extract instructions, and send each instruction as a separate transaction")
                .takes_value(false)
                .required(false)
            )
        )
        .get_matches();
    let config_file_path = get_config_or_default(&matches);
    process_matches(&matches, config_file_path).await?;
    Ok(())
}

// returns the value of the config file argument or the default
fn get_config_or_default(matches: &clap::ArgMatches) -> String {
    matches
        .value_of("config")
        .unwrap_or("config.yaml")
        .to_string()
}

async fn process_matches<'a>(
    matches: &clap::ArgMatches<'a>,
    config_file_path: String,
) -> Result<()> {
    match matches.subcommand() {
        ("config", Some(config_command)) => match config_command.subcommand() {
            ("new", Some(new_config)) => config::new_config(new_config, config_file_path),
            _ => invalid_subcommand("config"),
        },
        ("resign", Some(resign)) => {
            #[derive(
                Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
            )]
            #[serde(rename_all = "camelCase")]
            pub struct Root {
                pub method: String,
                pub jsonrpc: String,
                pub params: (String, Params),
                pub id: String,
            }

            #[derive(
                Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
            )]
            #[serde(rename_all = "camelCase")]
            pub struct Params {
                pub encoding: String,
                pub preflight_commitment: String,
            }

            let cfg = get_config(&config_file_path)?;
            let file_name = resign.value_of("input-file").unwrap();
            log::info!("reading transaction from file {}", file_name);
            let file_contents = std::fs::read_to_string(file_name)?;
            let file_data: Root = serde_json::from_str(&file_contents)?;
            log::info!("decoding transaction data");
            let txn_data = base64::decode(file_data.params.0)?;
            let tx: Transaction = match bincode::deserialize(&txn_data) {
                Ok(instruction) => instruction,
                Err(err) => {
                    return Err(anyhow!("failed to deserialize txn {:#?}", err));
                }
            };
            log::info!(
                "decompiling transaction which has {} instructions",
                tx.message.instructions.len()
            );
            let legacy_msg: solana_sdk::message::legacy::Message = tx.message().clone();
            let sanitized_msg = SanitizedMessage::Legacy(legacy_msg);
            let decompiled_instructions: Vec<Instruction> = sanitized_msg
                .decompile_instructions()
                .iter()
                .map(|ix| {
                    let program_id = *ix.program_id;
                    let data = ix.data.clone();
                    let accounts = ix
                        .accounts
                        .iter()
                        .map(|acct| AccountMeta {
                            pubkey: *acct.pubkey,
                            is_signer: acct.is_signer,
                            is_writable: acct.is_writable,
                        })
                        .collect::<Vec<AccountMeta>>();
                    Instruction {
                        program_id,
                        data: data.to_vec(),
                        accounts,
                    }
                })
                .collect();
            log::debug!("decompiled instructions {:#?}", decompiled_instructions);

            let rpcs: bsolana::rpc_utils::RPCs = From::from(&cfg.rpc_endpoints);
            let rpc = rpcs.get_rpc_client(false, None);
            let payer = cfg.payer_signer(Some(resign))?;

            let send_txn = |instructions: &[Instruction]| -> Result<()> {
                let mut tx = Transaction::new_with_payer(instructions, Some(&payer.pubkey()));
                log::info!("signing transaction with keypair {}", payer.pubkey());
                tx.sign(&vec![&*payer], rpc.get_latest_blockhash()?);
                log::info!("sending transaction");
                match rpc.send_and_confirm_transaction(&tx) {
                    Ok(sig) => log::info!("sent transaction {}", sig),
                    Err(err) => {
                        log::error!("failed to send transaction {:#?}", err);
                    }
                }
                Ok(())
            };

            if matches.is_present("one-by-one") {
                for ix in decompiled_instructions.iter() {
                    send_txn(&[ix.clone()])?;
                }
            } else {
                send_txn(&decompiled_instructions)?;
            }

            Ok(())
        }
        _ => invalid_command(),
    }
}

fn invalid_subcommand(command_group: &str) -> Result<()> {
    Err(anyhow!("invalid command found for group {}", command_group))
}

fn invalid_command() -> Result<()> {
    Err(anyhow!("invalid command found"))
}

pub fn get_solana_rpcs(config: &Configuration) -> bsolana::rpc_utils::RPCs {
    (&config.rpc_endpoints).into()
}
