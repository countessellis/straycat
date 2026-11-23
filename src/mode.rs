use std::{env::{args,Args},fmt,str::FromStr};

///////////// Mode

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Mode {
  Interactive,
  Batch,
  Help,
}

impl Default for Mode {
  fn default() -> Self {
    Mode::Batch
  }
}

impl FromStr for Mode {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self,Self::Err> {
    match s.to_lowercase().as_str() {
      "interactive" => Ok(Mode::Interactive),
      "batch"       => Ok(Mode::Batch),
      "help"        => Ok(Mode::Help),
      _             => Ok(Default::default()),
    }
  }
}

impl fmt::Display for Mode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Mode::Interactive => write!(f,"interactive"),
      Mode::Batch       => write!(f,"batch"),
      Mode::Help        => write!(f,"help"),
    }
  }
}

impl Mode {
  pub(crate) fn mode() -> Mode {
    log::info!("Getting mode from arguments.");
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--mode" => match args.next() {
          Some(mode) => match Mode::from_str(mode.as_str()) {
            Ok(mode) => return mode,
            Err(_)   => return Default::default(),
          },
          None        => {},
        },
        "--interactive"  => return Mode::Interactive,
        "--help" => return Mode::Help,
        _ => {},
      }
    }
    Default::default()
  }
}

