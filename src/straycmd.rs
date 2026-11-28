use std::{
  process::{Child,Command,Stdio},
};

use crate::strayconfig::*;

///////////// StrayCmd

pub struct StrayCmd;

impl StrayCmd {
  pub fn connect(cfg: &StrayConfig) -> Option<Child> {
    if let Some(ref execute) = cfg.execute {
      let mut args: Vec<&str> = execute.split(" ").collect();
      let command: &str = args.remove(0);
      match Command::new(command).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(cmd) => Some(cmd),
        Err(err)  => {
          log::error!("Failed to start command '{}': {}",command,err);
          None
        },
      }
    } else { None }
  }
}
