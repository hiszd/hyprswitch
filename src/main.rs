#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate strum; // 0.10.0
#[macro_use]
extern crate strum_macros; // 0.10.0
extern crate clap;
extern crate serde_derive;
extern crate serde_json;
use std::fs;
use std::net::SocketAddr;
use std::{env, ops::IndexMut};
use std::{error::Error, process::Command};
use std::{
  fmt,
  io::{Read, Write},
};
use std::{fs::read, thread};
use std::{
  io,
  os::unix::net::{UnixListener, UnixStream},
};
use std::{
  io::{BufRead, BufReader},
  path::Path,
};

use clap::Parser;
use serde_derive::{Deserialize, Serialize};
use serde_json::de::IoRead;
use serde_json::Value;
use strum::IntoEnumIterator;

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
  let monoutput = Command::new("/usr/bin/hyprctl")
    .arg("monitors")
    .arg("-j")
    .output()
    .unwrap();
  let monjson: Vec<Mon> = serde_json::from_slice(&monoutput.stdout).unwrap();
  MonList { monitors: monjson }
}

fn get_config() -> Result<Vec<Action>, io::Error> {
  let dir: &str =
    &(home::home_dir().unwrap().to_str().unwrap().to_owned() + "/.config/hyprswitch/");
  println!("{}", dir);

  fs::create_dir_all(dir).unwrap();
  let mut fil: String = String::new();
  match fs::read_to_string(dir.to_owned() + "config.json") {
    Ok(b) => fil = b,
    Err(e) => {
      return Err(e);
    }
  };
  println!("{}", fil);

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

fn parse_cmd(c: String, mons: &MonRtrn) -> Vec<String> {
  let mut cmds: Vec<String> = Vec::new();
  // TODO: return the string with the monitors name replaced
  // NOTE: maybe use regex builder of some kind?
  if let Some(ind) = c.find("${") {
    let mut cmd: String = c[ind + 2..c.len()].to_string();
    if let Some(inde) = cmd.find("}") {
      cmd = cmd[0..inde].to_string();
    }
    cmds.push(cmd);
  }
  cmds
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

fn check_config(config: &Vec<Action>, mon: &MonList) -> Result<(), io::Error> {
  config.iter().for_each(|a| {
    let mons: MonRtrn = parse_mons(&a.mons);
    if mon
      .monitors
      .iter()
      .any(|m| a.mons.contains(m.name.as_str()))
    {
      println!("id: {}", a.mons);
      a.cmds.iter().enumerate().for_each(|(i, c)| {
        let cmd = parse_cmd(c.to_owned(), &mons);
        cmd.iter().for_each(|e| {
        });
        println!("cmd{}: {} :: {:?}", i, c, cmd);
        // let mut cmd = Command::new("/usr/bin/hyprctl").arg(c);
      });
    }
  });
  Ok(())
}

fn main() -> Result<(), io::Error> {
  // get hyprland instance for socket path
  let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();

  // get socket path
  let filepath = "/tmp/hypr/".to_owned() + &hyprland_instance + "/.socket2.sock";
  let path = Path::new(&filepath);

  let mut config: Vec<Action> = get_config().unwrap();

  let mut mon = get_monitors();

  check_config(&config, &mon);

  loop {
    let mut strm = UnixStream::connect(path).unwrap();

    let stream = BufReader::new(strm);

    stream.lines().for_each(|e| {
      let arr = e.as_ref().unwrap().find(">>").unwrap();
      let x = &e.as_ref().unwrap()[0..arr];
      // let args: Vec<&str> = e.as_ref().unwrap()[(arr + 2)..].split(',').collect();
      match x {
        "monitorremoved" => {
          mon = get_monitors();
        }
        "monitoradded" => {
          mon = get_monitors();
        }
        _ => {}
      }
    })
  }
  Ok(())
}
