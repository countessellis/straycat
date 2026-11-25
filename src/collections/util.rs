use std::{
  io::{stdin,stdout,Write},
};

pub fn prompt(prompt: String, default: String) -> String {
  print!("{} ",prompt);
  stdout().flush().ok();
  let mut response = String::new();
  let _ = stdin().read_line(&mut response);
  if response.trim().is_empty() { response = default };
  response.trim_end().to_string()
}

