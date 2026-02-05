use std::process::Command;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum CommandsExecError {
  #[error("Commands Failed: {0:?}")]
  CommandsFailed(Vec<(String, String)>),
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

fn replace_home(s: String) -> String {
  s.replace(
    "~/",
    &(home::home_dir().unwrap().to_str().unwrap().to_owned() + "/"),
  )
  .to_owned()
}
pub fn exec_cmds(cmds: Vec<String>) -> Result<bool, CommandsExecError> {
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
      let failed_str =
        String::from_utf8(out.stdout).unwrap() + "\n***" + &String::from_utf8(out.stderr).unwrap();
      println!("Failed: {} - {}", e, failed_str);
      failed.push((e.clone(), failed_str));
    }
  });
  if success != cmds.len() {
    return Err(CommandsExecError::CommandsFailed(failed));
  }
  Ok(true)
}
