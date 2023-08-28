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
struct Action {
  mons: String,
  cmds: Vec<String>,
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
  pub fn findById(&self, id: i32) -> usize {
    let mut idx: usize = 0;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.id == id {
        idx = i;
      }
    });
    idx
  }
  pub fn findByName(&self, name: &str) -> usize {
    let mut idx: usize = 0;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.name == name {
        idx = i;
      }
    });
    idx
  }
}

fn get_monitors() -> MonList {
  let monoutput = Command::new("hyprctl")
    .arg("monitors")
    .arg("-j")
    .output()
    .unwrap();
  let monjson: Vec<Mon> = serde_json::from_slice(&monoutput.stdout).unwrap();
  MonList { monitors: monjson }
}

fn replace_home(s: String) -> String {
  s.replace("~/", &(home::home_dir().unwrap().to_str().unwrap().to_owned() + "/")).to_owned()
}

fn get_config() -> Result<Vec<Action>> {
  let dir: &str =
    &(home::home_dir().unwrap().to_str().unwrap().to_owned() + "/.config/hyprswitch/");

  fs::create_dir_all(dir).unwrap();
  let fil: String;
  match fs::read_to_string(dir.to_owned() + "config.json") {
    Ok(b) => fil = b,
    Err(e) => {
      return Err(e.into());
    }
  };

  let jsn: Vec<Action> = serde_json::from_str(&fil).unwrap();
  Ok(jsn)
}

#[derive(Debug, Clone)]
struct MonRtrn {
  required: Vec<String>,
  optional: Vec<String>,
}

impl MonRtrn {
  fn new() -> MonRtrn {
    MonRtrn {
      required: Vec::new(),
      optional: Vec::new(),
    }
  }
}

/// Function to replace variables with the values
fn parse_cmd(c: String, mons: &MonRtrn) -> Result<(String, Vec<String>)> {
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
      req.push(b.join(&mons.required[num - 1]));
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
      opt.push(b.join(&mons.optional[num - 1]));
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

fn parse_mons(m: &String) -> MonRtrn {
  let mut parts: MonRtrn = MonRtrn::new();
  m.split(',').for_each(|e| {
    if e.starts_with("&") {
      parts.optional.push(e[1..].to_string());
    } else {
      parts.required.push(e.to_string());
    }
  });
  parts
}

/// take the list of actions and the active monitors and determine the config to apply
fn determine_config(actions: Vec<Action>, mons: MonList) -> Result<Action> {
  let mut confident_action: (usize, usize) = (0, 0);
  // TODO: if monitor is detected and not on required or optional list then optmatch -= 1
  // TODO: if monitor is on the list of required monitors, but is not dected then reqmatch -= 1
  actions.iter().enumerate().for_each(|(i, e)| {
    let mut reqmatch: isize = 0;
    let mut optmatch: isize = 0;
    let montyp: MonRtrn = parse_mons(&e.mons);
    if montyp.required.iter().len() > 0 {
      montyp.required.iter().for_each(|e| {
        if mons.monitors.iter().find(|f| &f.name == e).is_some() {
          reqmatch += 1;
        } else {
          reqmatch -= 1;
        }
      });
      if reqmatch < 0 {
        reqmatch = 0;
      }
      if reqmatch as usize == montyp.required.iter().len() {
        if montyp.optional.iter().len() > 0 {
          montyp.optional.iter().for_each(|e| {
            if mons.monitors.iter().find(|f| &f.name == e).is_some() {
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
    println!(
      "{} || {}",
      format!("{:width$}", actions[i].mons, width = 20),
      format!("confidence: {}", optmatch + reqmatch,)
    );
  });
  let parsed_mons = parse_mons(&actions[confident_action.0].mons);
  println!(
    "\nSelected Config\n*** required: {:?} optional: {:?} ***\n\n",
    parsed_mons.required, parsed_mons.optional
  );
  let mut final_action = actions[confident_action.0].clone();
  final_action.cmds = final_action
    .cmds
    .iter()
    .map(|e| parse_cmd(e.to_string(), &parsed_mons).unwrap().0)
    .collect();
  Ok(final_action)
}

fn main() -> Result<()> {
  // get hyprland instance for socket path
  let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();

  // get socket path
  let filepath = "/tmp/hypr/".to_owned() + &hyprland_instance + "/.socket2.sock";
  let path = Path::new(&filepath);

  let mut mon: MonList = get_monitors();

  let mut conf = determine_config(get_config()?, mon.clone())?;

  exec_cmds(conf.cmds.clone()).unwrap();
  let mut pause = 0;

  loop {
    let strm = UnixStream::connect(path).unwrap();

    let stream = BufReader::new(strm);

    if pause > 10 {
      stream.lines().for_each(|e| {
        let arr = e.as_ref().unwrap().find(">>").unwrap();
        let x = &e.as_ref().unwrap()[0..arr];
        // let args: Vec<&str> = e.as_ref().unwrap()[(arr + 2)..].split(',').collect();
        match x {
          "monitorremoved" => {
            mon = get_monitors();
            conf = determine_config(get_config().unwrap(), mon.clone()).unwrap();
            exec_cmds(conf.cmds.clone()).unwrap();
          }
          "monitoradded" => {
            mon = get_monitors();
            conf = determine_config(get_config().unwrap(), mon.clone()).unwrap();
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
      failed.push((e.clone(), String::from_utf8(out.stdout).unwrap() + "\n***" + &String::from_utf8(out.stderr).unwrap()));
    }
  });
  if success != cmds.len() {
    println!("#ERROR Commands failed: {:?}", failed);
  }
  Ok(true)
}
