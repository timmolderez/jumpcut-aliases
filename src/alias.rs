use std::env;
use std::io::{Error, ErrorKind, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::fs;
use regex::{Regex,Captures};
use crate::utils::*;

pub struct Alias {
  alias: String,
  command: String,
  description: String,
  confirm: i8 // 0: no confirmation required ; 1: y/n confirmation ; 2: must confirm by entering alias name
}

impl Alias {
  /// Constructor
  /// 
  /// `alias`       : alias name
  /// `cmd`         : the command that this alias expands to; arguments are represented as ?1, ?2, etc. ; the present working directory is represented as $pwd
  /// `description` : an optional description of what this alias does
  /// `confirm`     : if true, a confirmation prompt is shown whenever executing this alias
  pub fn new(alias: &str, cmd: &str, description: &str, confirm: i8) -> Alias {
    return Alias{alias: alias.to_string(), command: cmd.to_string(), description: description.to_string(), confirm: confirm};
  }

  pub fn get_alias(&self) -> &str {return &self.alias;}
  pub fn get_command(&self) -> &str {return &self.command;}
  pub fn get_description(&self) -> &str {return &self.description;}

  pub fn update_description(self, description: &str) -> Alias {
    let description = if description=="-" {""} else {description};
    return Alias{description: description.to_string() , ..self};
  }
  pub fn update_confirm(self, confirm: i8) -> Alias {
    return Alias{confirm: confirm , ..self};
  }

  /// If true, a confirmation prompt is shown whenever executing this alias
  pub fn get_confirmation_level(&self) -> i8 {return self.confirm;}

  /// Reads an alias file
  /// 
  /// The format of an alias file is simple:
  /// - The name of the alias file must be the alias's name.
  /// - The contents of an alias file contains exactly 3 lines:
  ///   - the command that this alias expands to
  ///   - the alias's description (optional)
  ///   - the alias's options (e.g. "confirm=2")
  pub fn read(alias: &str, path: &Path) -> Result<Alias, Error> {
    let f = File::open(path)?;
    
    let f = BufReader::new(f);
    let confirm_default = 0;

    let lines:Result<Vec<String>, Error> = f.lines().collect();
    let lines = lines.unwrap_or_default();
    return match lines.len() {
      0 => Err(Error::new(ErrorKind::Other, format!("Empty or invalid alias file: {}", alias))),
      1 => Ok(Alias::new(alias, &*lines[0], "", confirm_default)),
      2 => Ok(Alias::new(alias, &*lines[0], &*lines[1], confirm_default)),
      3 => {
        let options = &lines[2];
        let split = options.split("=");
        let split_v: Vec<&str> = split.collect();
        if split_v.len()==2 && split_v[0] == "confirm" {
          let confirm = split_v[1].parse::<i8>().unwrap_or_default();
          return Ok(Alias::new(alias.clone(), &*lines[0], &*lines[1], confirm));
        } else {
          return Ok(Alias::new(alias, &*lines[0], &*lines[1], confirm_default));
        }
      }
      _ => Ok(Alias::new(alias.clone(), &*lines[0], &*lines[1], confirm_default)),
    };
  }

  /// Write an `Alias` to file
  /// 
  /// See alias::Alias::write() for information about the file format.
  pub fn write(&self, path: &Path) -> std::io::Result<()> {
    let data = self.command.clone() + "\n" + self.description.as_str() + "\nconfirm=" + &self.confirm.to_string();
    return fs::write(path, data);
  }

  /// "Execute" an alias using the given arguments
  /// 
  /// Actually, the command to be executed is only printed to the console!
  /// The wrapper script that calls Jumpcut is responsible for executing this command.
  /// This is deliberate because programs are not allowed to mess with the user's shell environment,
  /// e.g. by changing environment variables or changing the working directory. However, any shell script
  /// launched via `source` is allowed to do this.
  pub fn execute(&self, args:Vec<String>,
                 missing_arg_handler:&dyn Fn(&Alias, String) -> String) {
    // If the command contains "?pwd", this should be substituted for the current working directory
    let command_template = if self.command.ends_with("?pwd") {
      let abs_pwd = absolute_path(&env::current_dir().unwrap());
      let formatted_pwd = &format!("\"{}\"", abs_pwd)[..];
      self.command.replace("?pwd", formatted_pwd)
    } else {
      self.command.clone()
    };

    // Instantiate all arguments
    let re = Regex::new(r"\?[0-9]*").unwrap();
    let out = re.replace_all(&command_template[..], |caps: &Captures|{
      let re_match = caps.get(0).unwrap().as_str();
      let idx = re_match[1..2].parse::<usize>().unwrap();
      let val = match args.get(idx-1) {
        Some(v) => v,
        None => {
          let prompted_val = missing_arg_handler(self, re_match.to_string());
          return prompted_val;
        }
      };
      return format!("{}", val);
    });

    // Simply print the result
    println!("{}", out);
  }

  pub fn to_string(&self, width: usize) -> String {
    let flags = match self.get_confirmation_level() {
      0 => "",
      1 => "(Y/N confirmation)",
      2 => "(Explicit confirmation)",
      _ => ""
    };

    if self.description == "" {
      return format!("{: <w$}  {} {}", self.alias, self.command, flags, w=width);    
    } else {
      return format!("{: <w$}  {} {}\n{: <w$}  {}", self.alias, self.command, flags, "", self.description, w=width);
    }
  }
}

impl std::fmt::Display for Alias {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    return write!(f, "{}", self.to_string(self.alias.len()));   
  }
}
