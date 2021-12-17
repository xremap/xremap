use std::error::Error;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

pub fn load_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    let yaml = YamlLoader::load_from_str(&fs::read_to_string(&filename)?)?;
    // println!("{:#?}", yaml);
    match &yaml[..] {
        [yaml] => {
            Ok(parse_config(yaml))
        },
        arr => {
            Err(format!("The number of top-level elements must be 1, but got: {}", arr.len()).into())
        }
    }
}

fn parse_config(_yaml: &Yaml) -> Config {
    Config { modmap: vec![], keymap: vec![] }
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
