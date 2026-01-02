#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::format_in_format_args)]

extern crate clap;
extern crate serde_derive;
extern crate serde_json;
extern crate strum;
extern crate strum_macros;
use std::env;
use std::error;
use std::fs;
use std::os::unix::net::UnixStream;
use std::{
  io::{BufRead, BufReader},
  path::Path,
};

use crate::config::Config;

pub mod commands;
pub mod config;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

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

  let createdir = fs::create_dir_all(dir);
  if let Err(e) = createdir {
    return Err(e.into());
  }
  let fil = match fs::read_to_string(dir.to_owned() + "config.json") {
    Ok(b) => b,
    Err(e) => {
      return Err(e.into());
    }
  };

  let jsn = serde_json::from_str::<Config>(&fil);
  match jsn {
    Ok(j) => Ok(j),
    Err(e) => {
      println!("Could not parse config file: {}", e);
      Err(e.into())
    }
  }
}

fn main() -> Result<()> {
  // get hyprland instance for socket path
  let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
  let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();

  // get socket path
  let filepath = runtime_dir + "/hypr/" + &hyprland_instance + "/.socket2.sock";
  let path = Path::new(&filepath);

  let config = get_config()?;

  let monitors = commands::hyprctl::get_monitors();
  let aliases: Vec<config::alias::Alias> = config
    .aliases
    .iter()
    .map(|a| config::alias::Alias::from_configalias(a, &monitors).unwrap())
    .collect();
  let actions: Vec<config::action::Action> = config
    .actions
    .iter()
    .map(|a| config::action::Action::from_configaction(a, &aliases, &monitors))
    .collect();
  actions
    .iter()
    .for_each(|a| println!("{}: {}", a.configaction.mons, a.monandcon.1));
  let action = actions.iter().max_by_key(|a| a.monandcon.1);
  if let Some(a) = action {
    println!("\nSelected action:");
    println!("{}: {}\n", a.configaction.mons, a.monandcon.1);
    let _ = a.exec_cmds();
  }

  let mut pause = 0;

  loop {
    let strm = UnixStream::connect(path);
    if let Err(e) = &strm {
      println!("Couldn't connect to {filepath:?}, because of {e:?}")
    }

    let stream = BufReader::new(strm.unwrap());

    let loopconfig = config.clone();

    if pause > 100 {
      stream.lines().for_each(move |e| {
        let e = e.as_ref().unwrap();
        let arr = e.find(">>").unwrap();
        let x = &e[0..arr];
        // let args: Vec<&str> = e.as_ref().unwrap()[(arr + 2)..].split(',').collect();
        match x {
          "monitorremoved" | "monitorremovedv2" => {
            let monitors = commands::hyprctl::get_monitors();
            let aliases: Vec<config::alias::Alias> = loopconfig
              .aliases
              .iter()
              .map(|a| config::alias::Alias::from_configalias(a, &monitors).unwrap())
              .collect();
            let actions: Vec<config::action::Action> = loopconfig
              .actions
              .iter()
              .map(|a| config::action::Action::from_configaction(a, &aliases, &monitors))
              .collect();
            actions
              .iter()
              .for_each(|a| println!("{}: {}", a.configaction.mons, a.monandcon.1));
            let action = actions.iter().max_by_key(|a| a.monandcon.1);
            if let Some(a) = action {
              println!("\nSelected action:");
              println!("{}: {}\n", a.configaction.mons, a.monandcon.1);
              let _ = a.exec_cmds();
            }
          }
          "monitoradded" | "monitoraddedv2" => {
            let monitors = commands::hyprctl::get_monitors();
            let aliases: Vec<config::alias::Alias> = loopconfig
              .aliases
              .iter()
              .map(|a| config::alias::Alias::from_configalias(a, &monitors).unwrap())
              .collect();
            let actions: Vec<config::action::Action> = loopconfig
              .actions
              .iter()
              .map(|a| config::action::Action::from_configaction(a, &aliases, &monitors))
              .collect();
            actions
              .iter()
              .for_each(|a| println!("{}: {}", a.configaction.mons, a.monandcon.1));
            let action = actions.iter().max_by_key(|a| a.monandcon.1);
            if let Some(a) = action {
              println!("\nSelected action:");
              println!("{}: {}\n", a.configaction.mons, a.monandcon.1);
              let _ = a.exec_cmds();
            }
          }
          _ => {}
        }
      })
    } else {
      pause += 1;
    }
  }
}
