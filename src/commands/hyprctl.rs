use std::process::Command;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Aws {
  pub id: i32,
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorJson {
  pub id: i32,
  pub name: String,
  pub description: String,
  pub make: String,
  pub model: String,
  pub serial: String,
  pub width: u32,
  pub height: u32,
  pub refreshRate: f32,
  pub x: i32,
  pub y: i32,
  pub activeWorkspace: Aws,
  pub reserved: [u32; 4],
  pub scale: f32,
  pub transform: u32,
  pub focused: bool,
  pub dpmsStatus: bool,
  pub vrr: bool,
  pub disabled: bool,
}

pub struct Monitor {
  pub id: i32,
  pub name: String,
  pub description: String,
  pub make: String,
  pub model: String,
  pub serial: String,
  pub disabled: bool,
}

impl From<MonitorJson> for Monitor {
  fn from(monitor: MonitorJson) -> Monitor {
    Monitor {
      id: monitor.id,
      name: monitor.name,
      description: monitor.description,
      make: monitor.make,
      model: monitor.model,
      serial: monitor.serial,
      disabled: monitor.disabled,
    }
  }
}

pub fn get_monitors() -> Vec<MonitorJson> {
  let monoutput = Command::new("hyprctl")
    .arg("monitors")
    .arg("-j")
    .output()
    .unwrap();
  serde_json::from_slice::<Vec<MonitorJson>>(&monoutput.stdout).unwrap()
}
