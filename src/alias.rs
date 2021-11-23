use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::fs;
use regex::{Regex,Captures};
use crate::utils::*;

pub struct Alias {
  alias: String,
  command: String,
  description: String,
  confirm: i8, // 0: no confirmation required ; 1: y/n confirmation ; 2: must confirm by entering alias name
  storage_path: PathBuf
}

impl Alias {
  /// Constructor
  /// 
  /// `alias`       : alias name
  /// `cmd`         : the command that this alias expands to; arguments are represented as ?1, ?2, etc. ; the present working directory is represented as $pwd
  /// `description` : an optional description of what this alias does
  /// `confirm`     : if true, a confirmation prompt is shown whenever executing this alias
  /// `is_shared`   : if true, this alias is in the shared storage folder (rather than the default folder)
  pub fn new(alias: &str, cmd: &str, description: &str, confirm: i8, storage_path: PathBuf) -> Alias {
    return Alias{
      alias: alias.to_string(),
      command: cmd.to_string(),
      description: description.to_string(),
      confirm: confirm,
      storage_path: storage_path
    };
  }

  pub fn get_alias(&self) -> &str {return &self.alias;}
  pub fn get_command(&self) -> &str {return &self.command;}
  pub fn get_description(&self) -> &str {return &self.description;}
  pub fn get_file_path(&self) -> PathBuf {return self.storage_path.clone();}

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
    let path_buf = path.to_path_buf();
    
    let f = BufReader::new(f);
    let confirm_default = 0;

    let lines:Result<Vec<String>, Error> = f.lines().collect();
    let lines = lines.unwrap_or_default();
    return match lines.len() {
      0 => Err(Error::new(ErrorKind::Other, format!("Empty or invalid alias file: {}", alias))),
      1 => Ok(Alias::new(alias, &*lines[0], "", confirm_default, path_buf)),
      2 => Ok(Alias::new(alias, &*lines[0], &*lines[1], confirm_default, path_buf)),
      3 => {
        let options = &lines[2];
        let split = options.split("=");
        let split_v: Vec<&str> = split.collect();
        if split_v.len()==2 && split_v[0] == "confirm" {
          let confirm = split_v[1].parse::<i8>().unwrap_or_default();
          return Ok(Alias::new(alias.clone(), &*lines[0], &*lines[1], confirm, path_buf));
        } else {
          return Ok(Alias::new(alias, &*lines[0], &*lines[1], confirm_default, path_buf));
        }
      }
      _ => Ok(Alias::new(alias.clone(), &*lines[0], &*lines[1], confirm_default, path_buf)),
    };
  }

  /// Is this alias stored in the main storage folder?
  /// (.. or is it in the shared folder)
  pub fn is_in_main_storage(&self) -> bool {
    return self.storage_path.starts_with(alias_path());
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
  pub fn execute(&self, args:HashMap<String, String>,
                 missing_arg_handler:&dyn Fn(&Alias, &str) -> String) {
    // If the command contains "?pwd", this should be substituted for the current working directory
    let command_template = if self.command.ends_with("?pwd") {
      let abs_pwd = absolute_path(&env::current_dir().unwrap());
      let formatted_pwd = &format!("\"{}\"", abs_pwd)[..];
      self.command.replace("?pwd", formatted_pwd)
    } else {
      self.command.clone()
    };

    let instantiated_command = self.fill_in_parameters(
      command_template, args, missing_arg_handler, true);

    // Simply print the result
    println!("{}", instantiated_command);
  }

  /// Fill in the parameters of a command with `args`
  ///
  /// If `args` doesn't the value for a parameter, `missing_arg_handler` is called.
  /// If a parameter is intentially left blank, we'll pass a blank value if
  /// `full_instantiation` is true; otherwise we'll leave the parameter as-is.
  pub fn fill_in_parameters(&self, command: String, mut args:HashMap<String, String>,
                            missing_arg_handler:&dyn Fn(&Alias, &str) -> String,
                            full_instantiation: bool) -> String {
    let re = Regex::new(r"\?\[([A-Za-z0-9_]*)\]").unwrap();
    let out = re.replace_all(&command[..], |caps: &Captures|{
      let key = caps.get(1).unwrap().as_str();
      let val = match args.get(key) {
        Some(v) => v,
        None => {
          let mut prompted_val = missing_arg_handler(self, key);
          if prompted_val == "" && !full_instantiation {
            prompted_val = caps.get(0).unwrap().as_str().to_string();
          }
          // Add it to args, so we won't ask again if this argument occurs more than once
          args.insert(key.to_string(), prompted_val.clone());
          return prompted_val;
        }
      };
      return format!("{}", val);
    });

    return out.to_string();
  }

  pub fn to_string(&self, width: usize) -> String {
    let flags = match self.get_confirmation_level() {
      0 => "",
      1 => "(Y/N confirmation)",
      2 => "(Explicit confirmation)",
      _ => ""
    };

    let styled_alias = accent_style().apply_to(&self.alias);
    if self.description == "" {
      return format!("{: <w$}  {} {}", styled_alias, self.command, flags, w=width);
    } else {
      return format!("{: <w$}  {} {}\n{: <w$}  {}", styled_alias, self.command, flags, "", self.description, w=width);
    }
  }
}

impl std::fmt::Display for Alias {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    return write!(f, "{}", self.to_string(self.alias.len()));   
  }
}
