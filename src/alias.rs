use std::env;
use std::io::{Error, ErrorKind, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::fs;
use crate::utils::*;

pub struct Alias {
  alias: String,
  command: String,
  description: String,
  confirm: bool // If true, a confirmation prompt should be shown when executing this alias
}

impl Alias {
  pub fn new(alias: &str, cmd: &str, description: &str, confirm: bool) -> Alias {
    return Alias{alias: alias.to_string(), command: cmd.to_string(), description: description.to_string(), confirm: confirm};
  }

  pub fn get_alias(&self) -> &str {return &self.alias;}
  pub fn get_command(&self) -> &str {return &self.command;}
  pub fn get_description(&self) -> &str {return &self.description;}

  pub fn update_description(self, description: &str) -> Alias {
    return Alias{description: description.to_string() , ..self};
  }
  pub fn update_confirm(self, confirm: bool) -> Alias {
    return Alias{confirm: confirm , ..self};
  }

  pub fn must_confirm(&self) -> bool {return self.confirm;}

  pub fn read(alias: &str, path: &Path) -> Result<Alias, Error> {
    let f = File::open(path)?;
    let f = BufReader::new(f);
    let confirm_default = false;

    let lines:Result<Vec<String>, Error> = f.lines().collect();
    let lines = lines.unwrap_or_default();
    return match lines.len() {
      0 => Err(Error::new(ErrorKind::Other, format!("Empty or invalid alias file: {}", alias))),
      1 => Ok(Alias::new(alias, &*lines[0], "", confirm_default)),
      2 => Ok(Alias::new(alias, &*lines[0], &*lines[1], confirm_default)),
      3 => {
        let confirm = lines[2] == "confirm";
        return Ok(Alias::new(alias.clone(), &*lines[0], &*lines[1], confirm));
      }
      _ => Ok(Alias::new(alias.clone(), &*lines[0], &*lines[1], confirm_default)),
    };
  }

  pub fn write(&self, path: &Path) -> std::io::Result<()> {
    let confirm = if self.confirm {"\nconfirm"} else {""};
    let data = self.command.clone() + "\n" + self.description.as_str() + confirm;
    return fs::write(path, data);
  }

  pub fn execute(&self, args:Vec<String>) {
    // If the command contains "$prev", this should be substitued for the current working directory
    let command_template = if self.command.ends_with("$prev") {
      let abs_pwd = absolute_path(&env::current_dir().unwrap());
      let formatted_pwd = &format!("\"{}\"", abs_pwd)[..];
      self.command.replace("$prev", formatted_pwd)
    } else {
      self.command.clone()
    };

    // Instantiate all arguments
    let command_instance = args.as_slice().iter().enumerate().fold(command_template.clone(), 
      |acc:String, (i, arg)| acc.replace(&format!("${}", i), arg));

    // "Execute" the command by simply printing it. The execution is actually done by the wrapper script that calls Jumpcut.
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