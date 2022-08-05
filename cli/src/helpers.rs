use anyhow::Result;
use config::Configuration;
pub fn get_config(path: &str) -> Result<Configuration> {
    Configuration::load(path, true)
}
