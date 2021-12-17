use std::error::Error;
use std::fs;

pub fn load_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    let yaml = fs::read_to_string(&filename)?;
    println!("{}", yaml.len());
    return Ok(Config { modmap: vec![], keymap: vec![] })
}

#[derive(Debug)]
pub struct Config {
    pub modmap: Vec<Modmap>,
    pub keymap: Vec<Keymap>,
}

#[derive(Debug)]
pub struct Modmap {
    // TODO
}

#[derive(Debug)]
pub struct Keymap {
    // TODO
}
