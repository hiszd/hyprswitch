#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use serde_derive::{Deserialize, Serialize};

// structs for deserializing from json

/// Actions as defined in the config
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigAction {
  pub mons: String,
  pub cmds: Vec<String>,
}

/// Raw config file expectation for serialization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
  pub aliases: Option<Vec<String>>,
  pub actions: Vec<ConfigAction>,
}
