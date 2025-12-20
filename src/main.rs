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

#[allow(dead_code)]
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

impl error::Error for MonitorIndex {}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
  aliases: Option<Vec<String>>,
  actions: Vec<ConfigAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigAction {
  mons: String,
  cmds: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Action {
  mons: Mons,
  cmds: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Mons {
  available: MonSelected<Mon>,
  not_available: MonSelected<String>,
}

impl Action {
  fn from_configaction(c: ConfigAction, aliases: &[String]) -> Action {
    let replace_map: Vec<Alias> = parse_aliases(aliases.to_vec());
    let mons = get_monitors();
    let mut available = MonSelected::new_mon();
    let mut not_available: MonSelected<String> = MonSelected::new_string();
    c.mons.split(',').for_each(|e| {
      if let Some(f) = e.strip_prefix("&") {
        let mut f = f.to_string();
        let find = replace_map.iter().find(|a| a.name == f);
        if let Some(find1) = find {
          f = find1.value.clone();
        }
        let mon = mons.findByName(&f);
        if let Some(mon1) = mon
          && !mon1.disabled
        {
          available.optional.push(mon1);
        } else {
          not_available.optional.push(f.to_string());
        }
      } else {
        let mut f = e.to_string();
        let find = replace_map.iter().find(|a| a.name == f);
        if let Some(find1) = find {
          f = find1.value.clone()
        }
        let mon = mons.findByName(f.as_str());
        if let Some(mon1) = mon
          && !mon1.disabled
        {
          available.required.push(mon1);
        } else {
          not_available.required.push(f);
        }
      }
    });
    Action {
      mons: Mons {
        available,
        not_available,
      },
      cmds: c.cmds,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MonSelected<T> {
  required: Vec<T>,
  optional: Vec<T>,
}

trait GetMonString {
  fn required_string(&self) -> String;
  fn optional_string(&self) -> String;
}

impl MonSelected<Mon> {
  fn new_mon() -> MonSelected<Mon> {
    MonSelected {
      required: Vec::new(),
      optional: Vec::new(),
    }
  }
}

impl GetMonString for MonSelected<Mon> {
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

impl<String> MonSelected<String> {
  fn new_string() -> MonSelected<String> {
    MonSelected {
      required: Vec::new(),
      optional: Vec::new(),
    }
  }
}

impl GetMonString for MonSelected<String> {
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
  disabled: bool,
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
    idx.map(|idx1| self.monitors[idx1].clone())
  }
  pub fn findByName(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.name == name {
        idx = Some(i);
      }
    });
    idx.map(|idx1| self.monitors[idx1].clone())
  }
  pub fn findByModel(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.model == name {
        idx = Some(i);
      }
    });
    idx.map(|idx1| self.monitors[idx1].clone())
  }
  pub fn findByMake(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.make == name {
        idx = Some(i);
      }
    });
    idx.map(|idx1| self.monitors[idx1].clone())
  }
  pub fn findBySerial(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.serial == name {
        idx = Some(i);
      }
    });
    idx.map(|idx1| self.monitors[idx1].clone())
  }
  pub fn findByDescription(&self, name: &str) -> Option<Mon> {
    let mut idx: Option<usize> = None;
    self.monitors.iter().enumerate().for_each(|(i, e)| {
      if e.description == name {
        idx = Some(i);
      }
    });
    idx.map(|idx1| self.monitors[idx1].clone())
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

/// Function to replace variables with the values
fn parse_cmd(c: String, mons: &MonSelected<Mon>) -> Result<(String, Vec<String>)> {
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
    }
  });
  if req.is_empty() {
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
    }
  });
  if opt.is_empty() {
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
  actions.iter().fold(Vec::new(), |acc, e| {
    let mut a = acc.clone();
    let act = Action::from_configaction(e.clone(), &aliases);
    a.push(act);
    a
  })
}

struct Alias {
  name: String,
  value: String,
}

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

  let all_mons = get_monitors();
  aliases.iter().fold(Vec::new(), |mut acc, a| {
    let name = get_name(a.to_string());
    let mon = get_mon(name[1].to_string());
    match mon[0].as_str() {
      "model" => {
        let f = all_mons.findByModel(&mon[1]);
        if let Some(f1) = f {
          let m = Alias {
            name: name[0].to_string(),
            value: f1.name,
          };
          acc.push(m);
        }
        acc
      }
      "make" => {
        let f = all_mons.findByMake(&mon[1]);
        if let Some(f1) = f {
          let m = Alias {
            name: name[0].to_string(),
            value: f1.name,
          };
          acc.push(m);
        }
        acc
      }
      "serial" => {
        let f = all_mons.findBySerial(&mon[1]);
        if let Some(f1) = f {
          let m = Alias {
            name: name[0].to_string(),
            value: f1.name,
          };
          acc.push(m);
        }
        acc
      }
      "description" => {
        let f = all_mons.findByDescription(&mon[1]);
        if let Some(f1) = f {
          let m = Alias {
            name: name[0].to_string(),
            value: f1.name,
          };
          acc.push(m);
        }
        acc
      }
      _ => acc,
    }
  })
}

/// take the list of actions and the active monitors and determine the config to apply
fn determine_config(actions: Vec<Action>) -> Result<Action> {
  let mut confident_action: (usize, usize) = (0, 0);
  // TODO: if monitor is detected and not on required or optional list then optmatch -= 1
  // TODO: if monitor is on the list of required monitors, but is not dected then reqmatch -= 1
  actions.iter().enumerate().for_each(|(i, e)| {
    let mut reqmatch: isize = 0;
    let mut optmatch: isize = 0;
    if e.mons.available.required.iter().len() > 0 {
      reqmatch += e.mons.available.required.iter().len() as isize;
      reqmatch -= e.mons.not_available.required.iter().len() as isize;
      if reqmatch < 0 {
        reqmatch = 0;
      }
      if (reqmatch as usize
        == e.mons.available.required.iter().len() + e.mons.not_available.required.iter().len())
        && (e.mons.available.optional.iter().len() > 0)
      {
        optmatch += e.mons.available.optional.iter().len() as isize;
      }
    }
    if confident_action.1 <= (optmatch + reqmatch) as usize {
      confident_action.0 = i;
      confident_action.1 = (optmatch + reqmatch) as usize;
    }
    let reqstr = actions[i].mons.available.required_string()
      + &actions[i].mons.not_available.required_string();
    let optstr = actions[i].mons.available.optional_string()
      + &actions[i].mons.not_available.optional_string();
    println!(
      "{} || {}",
      format!("{:width$}", reqstr + &optstr, width = 20),
      format!("confidence: {}", optmatch + reqmatch,)
    );
  });
  let reqstr = actions[confident_action.0].mons.available.required_string()
    + &actions[confident_action.0]
      .mons
      .not_available
      .required_string();
  let optstr = actions[confident_action.0].mons.available.optional_string()
    + &actions[confident_action.0]
      .mons
      .not_available
      .optional_string();
  println!(
    "\nSelected Config\n*** required: {:?} optional: {:?} ***\n\n",
    reqstr, optstr
  );
  let mut final_action = actions[confident_action.0].clone();
  final_action.cmds = final_action
    .cmds
    .iter()
    .map(|e| {
      parse_cmd(e.to_string(), &final_action.mons.available)
        .unwrap()
        .0
    })
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
  let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();

  // get socket path
  let filepath = runtime_dir + "/hypr/" + &hyprland_instance + "/.socket2.sock";
  let path = Path::new(&filepath);

  let config = get_config()?;

  let transp = transpose_config(config.clone());

  let conf = determine_config(transp.clone())?;

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
    let strm = UnixStream::connect(path);
    if let Err(e) = &strm {
      println!("Couldn't connect to {filepath:?}, because of {e:?}")
    }

    let stream = BufReader::new(strm.unwrap());

    let loopconfig = config.clone();

    if pause > 100 {
      stream.lines().for_each(move |e| {
        let e = e.as_ref().unwrap();
        println!("#DEBUG {e}");
        let arr = e.find(">>").unwrap();
        let x = &e[0..arr];
        // let args: Vec<&str> = e.as_ref().unwrap()[(arr + 2)..].split(',').collect();
        match x {
          "monitorremoved" | "monitorremovedv2" => {
            // thread::sleep(Duration::from_secs(3));
            let transp = transpose_config(loopconfig.clone());
            let conf = determine_config(transp.clone()).unwrap();
            exec_cmds(conf.cmds.clone()).unwrap();
          }
          "monitoradded" | "monitoraddedv2" => {
            // thread::sleep(Duration::from_secs(3));
            let transp = transpose_config(loopconfig.clone());
            let conf = determine_config(transp.clone()).unwrap();
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
