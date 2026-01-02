use serde_derive::{Deserialize, Serialize};

use crate::commands::hyprctl::MonitorJson;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum ConfigAliasError {
  #[error("Invalid specifier: {0}\n Valid specifiers: id, name, description, make, model, serial")]
  InvalidSpecifier(String),
  #[error("Too many separators in name: {0}\n Usage: <alias>:<specifier>=<value>")]
  TooManySeparatorsInName(String),
  #[error("Too many equals in name: {0}\n Usage: <alias>:<specifier>=<value>")]
  TooManyEquals(String),
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

pub type ConfigAlias = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AliasSpecifier {
  Id,
  Name,
  Description,
  Make,
  Model,
  Serial,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alias {
  pub name: String,
  pub value: String,
  pub specifier: AliasSpecifier,
  pub matchedmonitor: Option<MonitorJson>,
}

impl Alias {
  pub fn from_configalias(
    calias: &ConfigAlias,
    monitors: &[MonitorJson],
  ) -> Result<Alias, ConfigAliasError> {
    let (alias, rem1) = &calias
      .split_once(':')
      .ok_or(ConfigAliasError::TooManyEquals(calias.clone()))?
      .to_owned();
    let (specifier, value) = rem1
      .split_once('=')
      .ok_or(ConfigAliasError::TooManySeparatorsInName(calias.clone()))?;
    let mut a = Alias {
      name: alias.to_owned().to_owned(),
      value: value.to_owned().to_owned(),
      specifier: match specifier {
        "id" => AliasSpecifier::Id,
        "name" => AliasSpecifier::Name,
        "description" => AliasSpecifier::Description,
        "make" => AliasSpecifier::Make,
        "model" => AliasSpecifier::Model,
        "serial" => AliasSpecifier::Serial,
        s => return Err(ConfigAliasError::InvalidSpecifier(s.to_owned())),
      },
      matchedmonitor: None,
    };
    if let Some(m) = a.match_mon(monitors) {
      a.matchedmonitor = Some(m);
    }
    Ok(a)
  }
  pub fn match_mon(&self, monitors: &[MonitorJson]) -> Option<MonitorJson> {
    let m = match self.specifier {
      AliasSpecifier::Id => monitors
        .iter()
        .find(|m| m.id == self.value.parse::<i32>().unwrap()),
      AliasSpecifier::Name => monitors.iter().find(|m| m.name == self.value),
      AliasSpecifier::Description => monitors.iter().find(|m| m.description == self.value),
      AliasSpecifier::Make => monitors.iter().find(|m| m.make == self.value),
      AliasSpecifier::Model => monitors.iter().find(|m| m.model == self.value),
      AliasSpecifier::Serial => monitors.iter().find(|m| m.serial == self.value),
    };
    m.cloned()
  }
}
