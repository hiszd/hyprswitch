#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use serde_derive::{Deserialize, Serialize};

pub mod action;
pub mod alias;

// structs for deserializing from json

/// Raw config file expectation for serialization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
  pub aliases: Vec<alias::ConfigAlias>,
  pub actions: Vec<action::ConfigAction>,
}
