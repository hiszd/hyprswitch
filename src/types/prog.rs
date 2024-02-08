#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use std::error;
use std::fs;
use std::process::Command;

use serde_derive::{Deserialize, Serialize};

use super::config::ConfigAction;
use crate::types::config::Config;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

// structs for use in the program

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Aws {
  pub id: i32,
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mon {
  pub id: i32,
  pub name: String,
  pub description: String,
  pub make: String,
  pub model: String,
  pub serial: String,
  pub width: u32,
  pub height: u32,
  pub refreshRate: f32,
  pub x: u32,
  pub y: u32,
  pub activeWorkspace: Aws,
  pub reserved: [u32; 4],
  pub scale: f32,
  pub transform: u32,
  pub focused: bool,
  pub dpmsStatus: bool,
  pub vrr: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonList {
  pub monitors: Vec<Mon>,
}

impl MonList {
  pub fn findById(&self, id: i32) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.id == id {
        idx = Some(i);
      }
    });
    if idx.is_some() {
      Some(self.monitors[idx.unwrap()].clone())
    } else {
      None
    }
  }
  pub fn findByName(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.name == name {
        idx = Some(i);
      }
    });
    if idx.is_some() {
      Some(self.monitors[idx.unwrap()].clone())
    } else {
      None
    }
  }
  pub fn findByModel(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.model == name {
        idx = Some(i);
      }
    });
    if idx.is_some() {
      Some(self.monitors[idx.unwrap()].clone())
    } else {
      None
    }
  }
  pub fn findByMake(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.make == name {
        idx = Some(i);
      }
    });
    if idx.is_some() {
      Some(self.monitors[idx.unwrap()].clone())
    } else {
      None
    }
  }
  pub fn findBySerial(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.serial == name {
        idx = Some(i);
      }
    });
    if idx.is_some() {
      Some(self.monitors[idx.unwrap()].clone())
    } else {
      None
    }
  }
  pub fn findByDescription(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.description == name {
        idx = Some(i);
      }
    });
    if idx.is_some() {
      Some(self.monitors[idx.unwrap()].clone())
    } else {
      None
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mons {
  pub found: ActionMon<Mon>,
  pub not_found: ActionMon<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alias {
  pub name: String,
  pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Commands {
  cmds: Vec<String>,
}

impl Commands {
  #[allow(dead_code)]
  pub fn new(cmds: Vec<String>) -> Commands { Commands { cmds } }
  #[allow(dead_code)]
  fn replace_home(s: String) -> String {
    s.replace(
      "~/",
      &(home::home_dir().unwrap().to_str().unwrap().to_owned() + "/"),
    )
    .to_owned()
  }
  #[allow(dead_code)]
  pub fn exec_cmds(cmds: Vec<String>) -> Result<bool> {
    let mut success = 0;
    let mut failed: Vec<(String, String)> = Vec::new();
    cmds.iter().for_each(|e| {
      let cmds = Commands::replace_home(e.clone());
      let mut cmd = Command::new("/bin/sh");
      cmd.arg("-c");
      cmd.arg(&cmds);
      let out = cmd.output().unwrap();

      if out.status.success() {
        success += 1;
      } else {
        failed.push((
          e.clone(),
          String::from_utf8(out.stdout).unwrap()
            + "\n***"
            + &String::from_utf8(out.stderr).unwrap(),
        ));
      }
    });
    if success != cmds.len() {
      println!("#ERROR Commands failed: {:?}", failed);
    }
    Ok(true)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
  pub mons: Mons,
  pub cmds: Commands,
}

impl Action {
  #[allow(dead_code)]
  pub fn get_monitors() -> MonList {
    let monoutput = Command::new("hyprctl")
      .arg("monitors")
      .arg("-j")
      .output()
      .unwrap();
    MonList {
      monitors: serde_json::from_slice(&monoutput.stdout).unwrap(),
    }
  }
  #[allow(dead_code)]
  fn parse_aliases(aliases: Vec<String>) -> Vec<Alias> {
    // the items in the array serve the following purposes: [extracted data, remainder]
    let get_name = |e: String| -> [String; 2] {
      let split_val = e.split(':').collect::<Vec<&str>>();
      assert!(split_val.len() == 2);
      [split_val[0].to_string(), split_val[1].to_string()]
    };

    // the items in the array serve the following purposes: [extracted data, remainder]
    let get_mon = |e: String| -> [String; 2] {
      let split_val = e.split('=').collect::<Vec<&str>>();
      assert!(split_val.len() == 2);
      [split_val[0].to_string(), split_val[1].to_string()]
    };

    let all_mons = Action::get_monitors();
    let new_aliases = aliases.iter().fold(Vec::new(), |mut acc, a| {
      let name = get_name(a.to_string());
      let mon = get_mon(name[1].to_string());
      match mon[0].as_str() {
        "model" => {
          let f = all_mons.findByModel(&mon[1]);
          if f.is_some() {
            let m = Alias {
              name: name[0].to_string(),
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        "make" => {
          let f = all_mons.findByMake(&mon[1]);
          if f.is_some() {
            let m = Alias {
              name: name[0].to_string(),
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        "serial" => {
          let f = all_mons.findBySerial(&mon[1]);
          if f.is_some() {
            let m = Alias {
              name: name[0].to_string(),
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        "description" => {
          let f = all_mons.findByDescription(&mon[1]);
          if f.is_some() {
            let m = Alias {
              name: name[0].to_string(),
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        _ => acc,
      }
    });
    new_aliases
  }
  #[allow(dead_code)]
  pub fn from_configaction(&self, c: ConfigAction, aliases: &Vec<String>) -> Action {
    let replace_map: Vec<Alias> = Action::parse_aliases(aliases.to_vec());
    let mons = Action::get_monitors();
    let mut found = ActionMon::new_mon();
    let mut not_found = ActionMon::new_string();
    c.mons.split(',').for_each(|e| {
      if e.starts_with("&") {
        let mut f = e[1..].to_string();
        let find = replace_map.iter().find(|a| a.name == f);
        if find.is_some() {
          f = find.unwrap().value.clone()
        }
        let mon = mons.findByName(f.as_str());
        if mon.is_some() {
          found.optional.push(mon.unwrap());
        } else {
          not_found.optional.push(f);
        }
      } else {
        let mut f = e.to_string();
        let find = replace_map.iter().find(|a| a.name == f);
        if find.is_some() {
          f = find.unwrap().value.clone()
        }
        let mon = mons.findByName(f.as_str());
        if mon.is_some() {
          found.required.push(mon.unwrap());
        } else {
          not_found.required.push(f);
        }
      }
    });
    Action {
      mons: Mons { found, not_found },
      cmds: Commands::new(c.cmds),
    }
  }
  #[allow(dead_code)]
  pub fn get_config() -> Result<Config> {
    let homedir = home::home_dir();
    if homedir.is_none() {
      return Err("Could not find home directory".into());
    }
    let hometmp = homedir.unwrap();
    let homestr = hometmp.to_str();
    if homestr.is_none() {
      return Err("could not get home directory string".into());
    }
    let dir: &str = &(homestr.unwrap().to_owned() + "/.config/hyprswitch/");

    let createdir = fs::create_dir_all(dir);
    if createdir.is_err() {
      return Err(createdir.unwrap_err().into());
    }
    let fil: String;
    match fs::read_to_string(dir.to_owned() + "config.json.new") {
      Ok(b) => fil = b,
      Err(e) => {
        return Err(e.into());
      }
    };

    let jsn = serde_json::from_str::<Config>(&fil);
    match jsn {
      Ok(j) => {
        return Ok(j);
      }
      Err(e) => {
        println!("Could not parse config file: {}", e);
        return Err(e.into());
      }
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionMon<T> {
  pub required: Vec<T>,
  pub optional: Vec<T>,
}

pub trait GetMonString {
  fn required_string(&self) -> String;
  fn optional_string(&self) -> String;
}

impl ActionMon<Mon> {
  pub fn new_mon() -> ActionMon<Mon> {
    ActionMon {
      required: Vec::new(),
      optional: Vec::new(),
    }
  }
}

impl GetMonString for ActionMon<Mon> {
  fn required_string(&self) -> String {
    self.required.iter().fold(String::new(), |mut acc, e| {
      acc.push_str((" ".to_owned() + e.name.as_str()).as_str());
      acc
    })
  }
  fn optional_string(&self) -> String {
    self.optional.iter().fold(String::new(), |mut acc, e| {
      acc.push_str((" ".to_owned() + e.name.as_str()).as_str());
      acc
    })
  }
}

impl<String> ActionMon<String> {
  pub fn new_string() -> ActionMon<String> {
    ActionMon {
      required: Vec::new(),
      optional: Vec::new(),
    }
  }
}

impl GetMonString for ActionMon<String> {
  fn required_string(&self) -> String {
    self.required.iter().fold(String::new(), |mut acc, e| {
      acc.push_str((" ".to_owned() + e.as_str()).as_str());
      acc
    })
  }
  fn optional_string(&self) -> String {
    self.optional.iter().fold(String::new(), |mut acc, e| {
      acc.push_str((" ".to_owned() + e.as_str()).as_str());
      acc
    })
  }
}
