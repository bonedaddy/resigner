use anyhow::Result;

use config::Configuration;

pub fn new_config(_matches: &clap::ArgMatches, config_file_path: String) -> Result<()> {
    let cfg = Configuration::default();
    cfg.save(config_file_path.as_str())?;
    Ok(())
}
