use serde_derive::{Deserialize, Serialize};

use crate::{commands::hyprctl::MonitorJson, config::alias::Alias};
use human_regex::{digit, named_capture, one_or_more, text};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigAction {
  pub mons: String,
  pub cmds: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionMonitor {
  pub name: String,
  pub optional: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
  pub configaction: ConfigAction,
  pub monandcon: (Vec<ActionMonitor>, isize),
  pub commands: Vec<String>,
}

// Optimization: Compile once and reuse [13, 14]
static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
  let pattern = text("${mons") + named_capture(one_or_more(digit()), "num") + text("}");

  pattern.to_regex() // Returns standard regex::Regex [10]
});

impl Action {
  pub fn from_configaction(
    caction: &ConfigAction,
    aliases: &[Alias],
    monitors: &[MonitorJson],
  ) -> Action {
    let mons: Vec<ActionMonitor> = caction.mons.split(",").fold(Vec::new(), |mut acc, s| {
      let n = s.strip_prefix("&");
      let p = n.unwrap_or(s).trim();
      let a = aliases.iter().find(|a| {
        if let Some(m) = a.matchedmonitor.clone() {
          a.name == p && !m.disabled
        } else {
          false
        }
      });
      if let Some(a1) = a {
        let name = a1.matchedmonitor.clone().unwrap().name.clone();
        acc.push(ActionMonitor {
          name,
          optional: n.is_some(),
        });
        acc
      } else if let Some(m) = monitors.iter().find(|m| m.name == p) {
        acc.push(ActionMonitor {
          name: m.name.clone(),
          optional: n.is_some(),
        });
        acc
      } else {
        acc
      }
    });
    // NOTE: this takes the commands, substitues the monitor names and returns the strings only if
    // a replacement was possible
    let cmds = {
      caction.cmds.iter().fold(Vec::new(), |mut acc, c| {
        let mut couldreplace = true;
        let result = RE.replace_all(c, |caps: &regex::Captures| {
          // Access the captured number string
          let num: usize = caps["num"].parse().unwrap();

          // Determine the dynamic replacement string
          if mons.len() >= num {
            mons[num - 1].name.clone()
          } else {
            couldreplace = false;
            "".to_owned()
          }
        });
        if couldreplace {
          acc.push(result.to_string());
        }
        acc
      })
    };
    let mut a = Action {
      configaction: caction.clone(),
      monandcon: (mons, 0),
      commands: cmds,
    };
    a.set_confidence();
    a
  }
  pub fn set_confidence(&mut self) {
    // TODO: Don't punish for optional monitors that are unavailable
    let cmons = self.configaction.mons.split(",").count();
    let smons = self.monandcon.0.len();
    self.monandcon.1 = if (cmons - smons) == 0 {
      smons as isize
    } else {
      (smons - cmons) as isize
    };
  }
  pub fn exec_cmds(&self) -> Result<bool, String> {
    match super::super::commands::exec::exec_cmds(self.commands.clone()) {
      Ok(e) => Ok(e),
      Err(e) => Err(e.to_string()),
    }
  }
}
