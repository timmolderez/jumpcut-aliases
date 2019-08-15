use std::env;
use std::io;
extern crate dialoguer;
extern crate dirs;
use std::path::PathBuf;
mod alias;
mod utils;
use utils::*;
use alias::Alias;
use dialoguer::{Confirmation, Select};
use std::fs;

/// Jumpcut - a command-line utility to quickly access frequently-used commands/folders
/// 
/// Run without any parameters to display the usage message.
fn main() -> Result<(),io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        usage();
        return Ok(());
    }

    fs::create_dir_all(config_path())?;

    let action = &args[1];
    match action.as_ref() {
        "is_exec_action" => {
            if args_ok(&args, 1) {
                println!("{}", !is_reserved_keyword(&args[2]));
            }
        }

        "list" => {
            return list_aliases();
        }

        "add" => {
            if args_ok(&args, 2) {
                let cmd = args[3..].join(" ");
                return add_alias(&args[2], &cmd);
            }
        }

        "addwd" => {
            if args_ok(&args, 2) {
                let abs_pwd = absolute_path(env::current_dir().unwrap());
                let cmd = args[3..].join(" ");
                return add_alias(&args[2], &format!("cd {};{}", abs_pwd, cmd));
            }
        }

        "addpath" => {
            if args_ok(&args, 2) {
                let abs_path = absolute_path(PathBuf::from(args[3].clone()));
                return add_alias(&args[2], &format!("cd {}", abs_path));
            }
        }

        "desc" => {
            if args_ok(&args, 2) {
                let desc = args[3..].join(" ");
                return add_description(&args[2], &desc);
            }
        }

        "rm" => {
            if args_ok(&args, 1) {
                return remove_alias(&args[2]);
            }
        },

        _ => {
            return find_and_exec_alias(action, args[2..].to_vec());
        }
    };

    return Ok(());
}

fn is_reserved_keyword(a: &String) -> bool {
    return a == "is_exec_action"
        || a == "list"
        || a == "add"
        || a == "addwd"
        || a == "addpath"
        || a == "desc"
        || a == "rm";
}

fn list_aliases() -> io::Result<()> {
    let path = config_path();
    let entries = path.read_dir()?;

    // Find the length of the longest alias; we need this for formatting the output
    let alias_len = entries.fold(0, |current_max, entry| {
        return match entry {
            Ok(file) => {
                let name_len = file.file_name().len();
                if name_len > current_max {
                    return name_len;
                } else {
                    return current_max;
                }
            }
            Err(_) => current_max,
        };
    });

    for entry in path.read_dir()? {
        let entry = entry?;
        let fname = entry.file_name();
        let path = config_path().join(fname);
        let fname_str = entry
            .file_name()
            .as_os_str()
            .to_os_string()
            .into_string()
            .unwrap();
        let al = Alias::read(&fname_str, &path)?;
        println!("{}", al.to_string(alias_len));
    }
    return Ok(());
}

fn find_and_exec_alias(alias: &String, args: Vec<String>) -> io::Result<()> {
    let path = config_path().join(alias);
    if path.exists() {
        exec_alias(alias, args);
    } else {
        let matches = config_path().read_dir()?.filter_map(|f| {
            let entry = f.unwrap();
            let fname_str = entry.file_name().as_os_str().to_os_string().into_string().unwrap();
            if fname_str.contains(alias) {
                return Some(fname_str);
            } else {
                return None;
            }
        });
        
        let match_vec: Vec<String> = matches.collect();
        match match_vec.len() {
            0 => (),
            1 => {
                exec_alias(&match_vec[0], args);
            },
            _ => {
                let selection = Select::new()
                    .default(0)
                    .items(&match_vec[..])
                    .interact_opt()
                    .unwrap();
                if let Some(selection) = selection {
                    exec_alias(&match_vec[selection], args);
                }
            }
        }
        
    }
    return Ok(());
}

fn exec_alias(alias: &String, args: Vec<String>) {
    let path = config_path().join(alias);
    match Alias::read(&alias, &path) {
        Ok(al) => al.execute(args),
        Err(e) => error(&e.to_string()),
    }
}

fn add_alias(alias: &String, cmd: &String) -> io::Result<()> {
    if is_reserved_keyword(alias) {
        error(&format!("\"{}\" cannot be used as an alias name; it is a reserved keyword.", alias));
        return Err(io::Error::new(io::ErrorKind::Other, "Reserved keyword."));
    }

    let al = Alias::new(alias.clone(), cmd.clone(), "".to_string());
    let path = config_path().join(alias);
    if path.exists() {
        match Confirmation::new()
            .with_text("Overwrite existing alias?")
            .interact()
        {
            Ok(_v) => {
                return al.write(&path);
            }
            Err(_v) => Ok(()),
        }
    } else {
        return al.write(&path);
    }
}

fn add_description(alias: &String, description: &String) -> io::Result<()> {
    let path = config_path().join(alias);
    let al = Alias::read(&alias, &path)?;
    let new_al = Alias::new(alias.clone(), al.get_command().clone(), description.clone());
    return new_al.write(&path);
}

fn remove_alias(alias: &String) -> io::Result<()> {
    let path = config_path().join(alias);
    if path.exists() {
        return fs::remove_file(path);
    } else {
        error(&format!("There is no alias named \"{}\".", alias));
        return Err(io::Error::new(io::ErrorKind::NotFound, "Alias not found."));
    }
}