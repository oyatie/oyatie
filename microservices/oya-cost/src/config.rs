#[derive(Debug, Default)]
pub struct Config;

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config)
}
