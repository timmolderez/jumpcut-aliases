use std::io::{Error, ErrorKind};
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::fs;

pub struct Alias {
  alias: String,
  command: String,
  description: String
}

impl Alias {
  pub fn new(alias: String, cmd: String, description: String) -> Alias {
    return Alias{alias: alias, command: cmd, description: description};
  }

  pub fn get_alias(&self) -> &String {return &self.alias;}
  pub fn get_command(&self) -> &String {return &self.command;}
  pub fn get_description(&self) -> &String {return &self.description;}

  pub fn read(alias: &String, path: &Path) -> Result<Alias, Error> {
    let f = File::open(path)?;
    let f = BufReader::new(f);

    let lines:Result<Vec<String>, Error> = f.lines().collect();
    let lines = lines.unwrap_or_default();

    return match lines.len() {
      0 => Err(Error::new(ErrorKind::Other, format!("Empty or invalid alias file: {}", alias))),
      1 => Ok(Alias::new(alias.clone(), lines[0].clone(), "".to_string())),
      2 => Ok(Alias::new(alias.clone(), lines[0].clone(), lines[1].clone())),
      _ => Ok(Alias::new(alias.clone(), lines[0].clone(), lines[1].clone())),
    };
  }

  pub fn write(&self, path: &Path) -> std::io::Result<()> {
    let data = self.command.clone() + "\n" + self.description.as_str();
    return fs::write(path, data);
  }

  pub fn execute(&self, args:Vec<String>) {
    let command_template = &self.command;
    let command_instance = args.as_slice().iter().enumerate().fold(command_template.clone(), 
      |acc:String, (i, arg)| acc.replace(&format!("${}", i), arg));
    println!("{}", command_instance);
  }

  pub fn to_string(&self, width: usize) -> String {
    if self.description == "" {
      return format!("{: <w$}  {}", self.alias, self.command, w=width);    
    } else {
      return format!("{: <w$}  {}\n{: <w$}  {}", self.alias, self.command, "", self.description, w=width);
    }
  }
}

impl std::fmt::Display for Alias {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    return write!(f, "{}", self.to_string(self.alias.len()));   
  }
}