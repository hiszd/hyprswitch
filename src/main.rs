#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]

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
use hyprswitch::types::config::*;
use hyprswitch::types::prog::*;
use serde_derive::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn determine_config(actions: Vec<Action>) -> Result<Action> {
  let mut confident_action: (usize, usize) = (0, 0);
  // TODO: if monitor is detected and not on required or optional list then optmatch -= 1
  // TODO: if monitor is on the list of required monitors, but is not dected then reqmatch -= 1
  actions.iter().enumerate().for_each(|(i, e)| {
    let mut reqmatch: isize = 0;
    let mut optmatch: isize = 0;
    if e.mons.found.required.iter().len() > 0 {
      reqmatch += e.mons.found.required.iter().len() as isize;
      reqmatch -= e.mons.not_found.required.iter().len() as isize;
      if reqmatch < 0 {
        reqmatch = 0;
      }
      if reqmatch as usize
        == (e.mons.found.required.iter().len() + e.mons.not_found.required.iter().len())
      {
        if e.mons.found.optional.iter().len() > 0 {
          optmatch += e.mons.found.optional.iter().len() as isize;
        }
      }
    }
    if confident_action.1 <= (optmatch + reqmatch) as usize {
      confident_action.0 = i;
      confident_action.1 = (optmatch + reqmatch) as usize;
    }
    let reqstr =
      actions[i].mons.found.required_string() + &actions[i].mons.not_found.required_string();
    let optstr =
      actions[i].mons.found.optional_string() + &actions[i].mons.not_found.optional_string();
    println!(
      "{} || {}",
      format!("{:width$}", reqstr + &optstr, width = 20),
      format!("confidence: {}", optmatch + reqmatch,)
    );
  });
  let reqstr = actions[confident_action.0].mons.found.required_string()
    + &actions[confident_action.0].mons.not_found.required_string();
  let optstr = actions[confident_action.0].mons.found.optional_string()
    + &actions[confident_action.0].mons.not_found.optional_string();
  println!(
    "\nSelected Config\n*** required: {:?} optional: {:?} ***\n\n",
    reqstr, optstr
  );
  let mut final_action = actions[confident_action.0].clone();
  final_action.cmds = final_action
    .cmds
    .iter()
    .map(|e| {
      parse_cmd(e.to_string(), &final_action.mons.found)
        .unwrap()
        .0
    })
    .collect();
  Ok(final_action)
}

fn main() -> Result<()> {
  // get hyprland instance for socket path
  let hyprland_instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();

  // get socket path
  let filepath = "/tmp/hypr/".to_owned() + &hyprland_instance + "/.socket2.sock";
  let path = Path::new(&filepath);

  let config = Action::get_config()?;

  let transp = config.actions.fold(Vec::new(), |mut acc, e| {
    acc.push(Action::from_configaction(e, config.aliases.clone()));
    acc
  });

  let mut conf = determine_config(transp.clone())?;

  // println!(
  //   "#DEBUG \n{}",
  //   conf
  //     .cmds
  //     .iter()
  //     .fold(String::new(), |acc, e| acc + &e + "\n ")
  // );
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
            let mon = Action::get_monitors();
            conf = determine_config(transp.clone()).unwrap();
            exec_cmds(conf.cmds.clone()).unwrap();
          }
          "monitoradded" => {
            let mon = Action::get_monitors();
            conf = determine_config(transp.clone()).unwrap();
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
