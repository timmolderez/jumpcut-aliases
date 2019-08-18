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
  /// Constructor
  /// 
  /// `alias`       : alias name
  /// `cmd`         : the command that this alias expands to; arguments are represented as $1, $2, etc. ; the present working directory is represented as $pwd
  /// `description` : an optional description of what this alias does
  /// `confirm`     : if true, a confirmation prompt is shown whenever executing this alias
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

  /// If true, a confirmation prompt is shown whenever executing this alias
  pub fn must_confirm(&self) -> bool {return self.confirm;}

  /// Reads an alias file
  /// 
  /// The format of an alias file is simple:
  /// - The name of the alias file must be the alias's name.
  /// - The contents of an alias file contains exactly 3 lines:
  ///   - the command that this alias expands to
  ///   - the alias's description (optional)
  ///   - the alias's options (e.g. "confirm")
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

  /// Write an `Alias` to file
  /// 
  /// See alias::Alias::write() for information about the file format.
  pub fn write(&self, path: &Path) -> std::io::Result<()> {
    let confirm = if self.confirm {"\nconfirm"} else {""};
    let data = self.command.clone() + "\n" + self.description.as_str() + confirm;
    return fs::write(path, data);
  }

  /// "Execute" an alias using the given arguments
  /// 
  /// Actually, the command to be executed is only printed to the console.
  /// The wrapper script that calls Jumpcut is responsible for executing this command.
  /// This is deliberate because programs are not allowed to mess with the user's shell environment,
  /// e.g. by changing environment variables or changing the working directory. However, any shell script
  /// launched via `source` is allowed to do this.
  pub fn execute(&self, args:Vec<String>) {
    // If the command contains "$prev", this should be substitued for the current working directory
    let command_template = if self.command.ends_with("$pwd") {
      let abs_pwd = absolute_path(&env::current_dir().unwrap());
      let formatted_pwd = &format!("\"{}\"", abs_pwd)[..];
      self.command.replace("$prev", formatted_pwd)
    } else {
      self.command.clone()
    };

    // Instantiate all arguments
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