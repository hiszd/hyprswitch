#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate clap;
extern crate serde_derive;
extern crate serde_json;
extern crate strum;
extern crate strum_macros;
use std::env;
use std::error;
use std::fmt;
use std::fs;
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::{
  io::{BufRead, BufReader},
  path::Path,
};

use human_regex::{digit, one_or_more, text};
use serde_derive::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct MonitorIndex;

impl fmt::Display for MonitorIndex {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "monitor variable asked for a monitor index that doesn't exist",
    )
  }
}

impl error::Error for MonitorIndex {
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
  aliases: Option<Vec<serde_json::Value>>,
  actions: Vec<ConfigAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigAction {
  mons: String,
  cmds: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Action {
  mons: ActionMons,
  cmds: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ActionMons {
  required: Vec<Mon>,
  optional: Vec<Mon>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Aws {
  id: i32,
  name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mon {
  id: i32,
  name: String,
  description: String,
  make: String,
  model: String,
  serial: String,
  width: u32,
  height: u32,
  refreshRate: f32,
  x: u32,
  y: u32,
  activeWorkspace: Aws,
  reserved: [u32; 4],
  scale: f32,
  transform: u32,
  focused: bool,
  dpmsStatus: bool,
  vrr: bool,
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

fn get_monitors() -> MonList {
  let monoutput = Command::new("hyprctl")
    .arg("monitors")
    .arg("-j")
    .output()
    .unwrap();
  MonList {
    monitors: serde_json::from_slice(&monoutput.stdout).unwrap(),
  }
}

fn replace_home(s: String) -> String {
  s.replace(
    "~/",
    &(home::home_dir().unwrap().to_str().unwrap().to_owned() + "/"),
  )
  .to_owned()
}

fn get_config() -> Result<Config> {
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
  println!("Looking for config in {}", dir);

  let createdir = fs::create_dir_all(dir);
  if createdir.is_err() {
    return Err(createdir.unwrap_err().into());
  }
  let fil: String;
  match fs::read_to_string(dir.to_owned() + "config.json") {
    Ok(b) => fil = b,
    Err(e) => {
      return Err(e.into());
    }
  };

  let jsn = serde_json::from_str::<Config>(&fil);
  match jsn {
    Ok(jsn) => {
      return Ok(jsn);
    }
    Err(e) => {
      return Err(e.into());
    }
  }
}

/// Function to replace variables with the values
fn parse_cmd(c: String, mons: &ActionMons) -> Result<(String, Vec<String>)> {
  let mut cmds: Vec<String> = Vec::new();
  // TODO: return the string with the monitors name replaced
  let num_regex = one_or_more(digit());
  let req_regex = text("${mons") + one_or_more(digit()) + text("}");
  let opt_regex = text("${&mons") + one_or_more(digit()) + text("}");
  let mut req: Vec<String> = Vec::new();
  req_regex.to_regex().find_iter(&c).for_each(|e| {
    cmds.push(e.as_str().to_string());
    let b: Vec<&str> = c.split(e.as_str()).collect();
    let num: usize = num_regex
      .to_regex()
      .find(e.as_str())
      .unwrap()
      .as_str()
      .parse()
      .unwrap();
    if mons.required.len() >= num {
      req.push(b.join(&mons.required[num - 1].name));
    } else {
      println!(
        "monitor variable {} asks for {}, but there are only {} monitors",
        e.as_str(),
        num,
        mons.required.len()
      );
      Err::<(String, Vec<String>), MonitorIndex>(MonitorIndex).unwrap();
    }
  });
  if req.len() == 0 {
    req.push(c);
  }
  let req_string = req.join(",");

  let mut opt: Vec<String> = Vec::new();

  opt_regex.to_regex().find_iter(&req_string).for_each(|e| {
    cmds.push(e.as_str().to_string());
    let b: Vec<&str> = req_string.split(e.as_str()).collect();
    let num: usize = num_regex
      .to_regex()
      .find(e.as_str())
      .unwrap()
      .as_str()
      .parse()
      .unwrap();
    if mons.optional.len() >= num {
      opt.push(b.join(&mons.optional[num - 1].name));
    } else {
      println!(
        "monitor variable {} asks for {}, but there are only {} monitors",
        e.as_str(),
        num,
        mons.optional.len()
      );
      Err::<(String, Vec<String>), MonitorIndex>(MonitorIndex).unwrap();
    }
  });
  if opt.len() == 0 {
    opt.push(req_string);
  }

  Ok((opt.join(""), cmds))
}

fn transpose_config(conf: Config) -> Vec<Action> {
  let mut aliases = Vec::new();
  if let Some(a) = conf.aliases {
    aliases = a;
  }
  let actions: Vec<ConfigAction> = conf.actions;
  let newacts = actions.iter().fold(Vec::new(), |acc, e| {
    let mut a = acc.clone();
    let mons = parse_mons(&e.mons, &aliases);
    println!("Parsed Monitors: {:?}", mons);
    a.push(Action {
      cmds: e.cmds.clone(),
      mons,
    });
    a
  });
  newacts
}

struct Alias {
  name: String,
  value: String,
}

fn parse_aliases(aliases: Vec<serde_json::Value>) -> Vec<Alias> {
  let all_mons = get_monitors();
  let new_aliases = aliases.iter().fold(Vec::new(), |mut acc, a| {
    if a.is_object() {
      let b = a.as_object().unwrap();
      assert!(b.keys().len() == 1);
      assert!(b.values().len() == 1);
      let name = b.keys().next().unwrap().to_string();
      let value = b.values().next().unwrap().to_string();
      let split_val = value.split('=').collect::<Vec<&str>>();
      match *split_val.get(0).unwrap_or(&"") {
        "model" => {
          let f = all_mons.findByModel(split_val.get(1).unwrap_or(&""));
          if f.is_some() {
            let m = Alias {
              name,
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        "make" => {
          let f = all_mons.findByMake(split_val.get(1).unwrap_or(&""));
          if f.is_some() {
            let m = Alias {
              name,
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        "serial" => {
          let f = all_mons.findBySerial(split_val.get(1).unwrap_or(&""));
          if f.is_some() {
            let m = Alias {
              name,
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        "description" => {
          let f = all_mons.findByDescription(split_val.get(1).unwrap_or(&""));
          if f.is_some() {
            let m = Alias {
              name,
              value: f.unwrap().name.into(),
            };
            acc.push(m);
          }
          acc
        }
        _ => acc,
      }
    } else {
      acc
    }
  });
  new_aliases
}

/// parse the string into required and optional monitors
fn parse_mons(m: &String, aliases: &Vec<serde_json::Value>) -> ActionMons {
  let replace_map: Vec<Alias> = parse_aliases(aliases.to_vec());
  let mons = get_monitors();
  let mut required: Vec<Mon> = Vec::new();
  let mut optional: Vec<Mon> = Vec::new();
  m.split(',').for_each(|e| {
    if e.starts_with("&") {
      let mut f = e[1..].to_string();
      let find = replace_map.iter().find(|a| a.name == f);
      if find.is_some() {
        f = find.unwrap().value.clone()
      }
      let mon = mons.findByName(f.as_str());
      if mon.is_some() {
        optional.push(mon.unwrap());
      }
    } else {
      let mut f = e.to_string();
      let find = replace_map.iter().find(|a| a.name == f);
      if find.is_some() {
        f = find.unwrap().value.clone()
      }
      let mon = mons.findByName(f.as_str());
      if mon.is_some() {
        required.push(mon.unwrap());
      }
    }
  });
  ActionMons { required, optional }
}

/// take the list of actions and the active monitors and determine the config to apply
fn determine_config(actions: Vec<Action>, mons: MonList) -> Result<Action> {
  let mut confident_action: (usize, usize) = (0, 0);
  // TODO: if monitor is detected and not on required or optional list then optmatch -= 1
  // TODO: if monitor is on the list of required monitors, but is not dected then reqmatch -= 1
  actions.iter().enumerate().for_each(|(i, e)| {
    let mut reqmatch: isize = 0;
    let mut optmatch: isize = 0;
    if e.mons.required.iter().len() > 0 {
      e.mons.required.iter().for_each(|e| {
        if mons.monitors.iter().find(|f| f.name == e.name).is_some() {
          reqmatch += 1;
        } else {
          reqmatch -= 1;
        }
      });
      if reqmatch < 0 {
        reqmatch = 0;
      }
      if reqmatch as usize == e.mons.required.iter().len() {
        if e.mons.optional.iter().len() > 0 {
          e.mons.optional.iter().for_each(|e| {
            if mons.monitors.iter().find(|f| f.name == e.name).is_some() {
              optmatch += 1;
            }
          });
        }
      }
    }
    if confident_action.1 <= (optmatch + reqmatch) as usize {
      confident_action.0 = i;
      confident_action.1 = (optmatch + reqmatch) as usize;
    }
    let reqstr = actions[i]
      .mons
      .required
      .iter()
      .fold(String::new(), |mut acc, e| {
        acc.push_str((" ".to_owned() + e.name.as_str()).as_str());
        acc
      });
    let optstr = actions[i]
      .mons
      .optional
      .iter()
      .fold(String::new(), |mut acc, e| {
        acc.push_str((" ".to_owned() + e.name.as_str()).as_str());
        acc
      });
    println!(
      "{} || {}",
      format!("{:width$}", reqstr + &optstr, width = 20),
      format!("confidence: {}", optmatch + reqmatch,)
    );
  });
  let reqstr =
    actions[confident_action.0]
      .mons
      .required
      .iter()
      .fold(String::new(), |mut acc, e| {
        acc.push_str((" ".to_owned() + e.name.as_str()).as_str());
        acc
      });
  let optstr =
    actions[confident_action.0]
      .mons
      .optional
      .iter()
      .fold(String::new(), |mut acc, e| {
        acc.push_str((" ".to_owned() + e.name.as_str()).as_str());
        acc
      });
  println!(
    "\nSelected Config\n*** required: {:?} optional: {:?} ***\n\n",
    reqstr, optstr
  );
  let mut final_action = actions[confident_action.0].clone();
  final_action.cmds = final_action
    .cmds
    .iter()
    .map(|e| parse_cmd(e.to_string(), &final_action.mons).unwrap().0)
    .collect();
  Ok(final_action)
}

fn exec_cmds(cmds: Vec<String>) -> Result<bool> {
  let mut success = 0;
  let mut failed: Vec<(String, String)> = Vec::new();
  cmds.iter().for_each(|e| {
    let cmds = replace_home(e.clone());
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c");
    cmd.arg(&cmds);
    let out = cmd.output().unwrap();

    if out.status.success() {
      success += 1;
    } else {
      failed.push((
        e.clone(),
        String::from_utf8(out.stdout).unwrap() + "\n***" + &String::from_utf8(out.stderr).unwrap(),
      ));
    }
  });
  if success != cmds.len() {
    println!("#ERROR Commands failed: {:?}", failed);
  }
  Ok(true)
}

fn main() -> Result<()> {
  // get hyprland instance for socket path
  let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();

  // get socket path
  let filepath = "/tmp/hypr/".to_owned() + &hyprland_instance + "/.socket2.sock";
  let path = Path::new(&filepath);

  let mut mon: MonList = get_monitors();

  let config = get_config()?;

  let transp = transpose_config(config);

  let mut conf = determine_config(transp.clone(), mon.clone())?;

  exec_cmds(conf.cmds.clone()).unwrap();
  let mut pause = 0;

  loop {
    let strm = UnixStream::connect(path).unwrap();

    let stream = BufReader::new(strm);

    if pause > 100 {
      stream.lines().for_each(|e| {
        let arr = e.as_ref().unwrap().find(">>").unwrap();
        let x = &e.as_ref().unwrap()[0..arr];
        // let args: Vec<&str> = e.as_ref().unwrap()[(arr + 2)..].split(',').collect();
        match x {
          "monitorremoved" => {
            mon = get_monitors();
            conf = determine_config(transp.clone(), mon.clone()).unwrap();
            exec_cmds(conf.cmds.clone()).unwrap();
          }
          "monitoradded" => {
            mon = get_monitors();
            conf = determine_config(transp.clone(), mon.clone()).unwrap();
            exec_cmds(conf.cmds.clone()).unwrap();
          }
          _ => {}
        }
      })
    } else {
      pause += 1;
    }
  }
}
