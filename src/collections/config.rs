use std::{
  collections::HashMap,
  env::{args,Args},
  fmt,
  fs,
  path::Path,
};

use super::util;

///////////// Config

#[derive(Debug, Clone)]
pub struct Config {
  pub config: HashMap<String,String>,
}

impl fmt::Display for Config {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let key_width: usize = self.config.keys().map(|k| k.len()).max().unwrap_or(0);
    let config: String = self.config.iter().map(|(parameter,value)| { let key = format!("    {}:",parameter); format!("{:<width$} {}",key,value, width=key_width) }).collect::<Vec<String>>().join("\n");
    write!(f,
"
  -----------------------------------
              Configuration
  -----------------------------------

{}

  -----------------------------------
",
      config,
    )
  }
}

impl Config {
  pub fn new(config_dir: &String, config_file: &String) -> Config {
    let mut config: Self = Self {
      config: HashMap::new(),
    };
    config.from_file(config_dir.clone(),config_file.clone());
    config.from_args();
    config
  }

  pub fn get(&self, parameter: &str) -> String {
    self.config.get(&parameter.to_string()).cloned().unwrap_or_default()
  }

  pub fn from_hashmap(defaults: &HashMap<String,String>) -> Config {
    log::debug!("Loading config with defaults.");
    let config: Self = Self {
      config: defaults.clone()
    };
    config
  }

  pub fn to_string(&self) -> String {
    log::debug!("Converting config to string.");
    self.config.iter().map(|(parameter,value)| format!("{}: {}",parameter,value)).collect::<Vec<String>>().join("\n")
  }

  pub fn from_string(&mut self, source: &String) {
    log::debug!("Reading config from provided string.");
    for line in source.split("\n") {
      let pair: Vec<&str> = line.split(":").collect();
      if pair.len() > 1 {
        let value: String = pair[1..].join(":").trim_start().to_string();
        log::debug!("Saving parameter '{}' with value '{}' to config.",pair[0],value);
        self.config.insert(pair[0].to_string(),value);
      }
    }
  } 

  pub fn from_file(&mut self, config_dir: String, config_file: String) {
    log::debug!("Finding default config directory and default config file.");
    let mut config_file = format!("{}/{}",config_dir,config_file);
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--config" => {
          match args.next() {
            Some(config_path) => config_file = config_path,
            None              => {},
          }
        },
        _ => {},
      }
    }
    let config_file: String = match fs::exists(&config_file) {
      Ok(true) => config_file.clone(),
      Ok(false) => {
        println!("Config file {} does not exist.",config_file);
        let to_file: bool = util::prompt(format!("Write config to new file at {}? (true/false, default false)",config_file),"false".to_string()).parse().unwrap_or(false);
        if to_file {
          match self.save_file(&config_file) {
            Ok(message) => {
              log::info!("{}",message);
              println!("{}",message);
            },
            Err(err) => log::error!("{}",err),
          }
        }
        return;
      },
      Err(err) => {
        log::error!("Error testing if {} exists: {}",config_file,err);
        return;
      }
    };
    log::debug!("Attempting to read config from {}, will ignore on error.",config_file);
    let config: String = fs::read_to_string(config_file).unwrap_or_default();
    self.from_string(&config);
  } 
  
  pub fn save_file(&mut self, config_file: &String) -> Result<String,String> {
    if let Some(parent) = Path::new(config_file).parent() && !parent.as_os_str().is_empty() && let Err(err) = fs::create_dir_all(parent) {
      return Err(format!("Failed to create parent directories for {}: {}",config_file,err));
    }
    match fs::write(&config_file,Self::to_string(&self)) {
      Ok(()) => Ok(format!("Outputted config to {}",config_file)),
      Err(err) => Err(format!("Failed to output config to {}: {}",config_file,err))
    }
  }

  pub fn from_args(&mut self) {
    log::debug!("Reading config from arguments.");
    let config = &mut self.config;
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        option => {
          if option.starts_with("--") {
            if let Some(option) = option.strip_prefix("--") {
              match args.next() {
                Some(value) => { config.insert(option.to_string(),value.to_string()); },
                None => {},
              }
            }
          }
        },
      }
    }
  }
}

