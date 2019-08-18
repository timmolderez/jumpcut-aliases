use std::env;
use std::io;
extern crate dialoguer;
extern crate dirs;
use std::path::PathBuf;
pub mod alias;
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

    fs::create_dir_all(alias_path())?;

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
                let abs_pwd = absolute_path(&env::current_dir().unwrap());
                let cmd = args[3..].join(" ");
                return add_alias(&args[2], &format!("cd \"{}\";{};cd $pwd", abs_pwd, cmd));
            }
        }

        "addpath" => {
            if args_ok(&args, 2) {
                let path = args[3..].join(" ");
                let abs_path = absolute_path(&PathBuf::from(path));
                return add_alias(&args[2], &format!("cd \"{}\"", abs_path));
            }
        }

        "desc" => {
            if args_ok(&args, 2) {
                let desc = args[3..].join(" ");
                return add_description(&args[2], &desc);
            }
        }

        "confirm" => {
            if args_ok(&args, 2) {
                return set_confirmation(&args[2], &args[3]=="true");
            }
        }

        "rm" => {
            if args_ok(&args, 1) {
                remove_alias(&args[2])?;
            }
        },

        _ => {
            return find_and_exec_alias(action, args[2..].to_vec());
        }
    };

    return Ok(());
}

/// Is `action` a reserved keyword or an alias name?
fn is_reserved_keyword(action: &str) -> bool {
    return action == "is_exec_action"
        || action == "list"
        || action == "add"
        || action == "addwd"
        || action == "addpath"
        || action == "desc"
        || action == "confirm"
        || action == "rm";
}

/// Displays a list of all aliases, together with their command and description
fn list_aliases() -> io::Result<()> {
    let path = alias_path();
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
        let path = alias_path().join(fname);
        let fname_str = entry.file_name().into_string().unwrap();
        let al = Alias::read(&fname_str, &path)?;
        println!("{}", al.to_string(alias_len));
    }
    return Ok(());
}

/// Given (part) of an alias name, find any matches and execute it
/// 
/// If there are multiple matches, ask the user to choose one.
fn find_and_exec_alias(alias: &str, args: Vec<String>) -> io::Result<()> {
    let path = alias_path().join(alias);
    if path.exists() {
        // If there's an exact match of the user's input
        exec_alias(alias, args)?;
    } else {
        // Otherwise, look for any aliases that contain the user's input
        let matches = alias_path().read_dir()?.filter_map(|f| {
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
            0 => {
                error("no matching aliases found.");
                exec_nothing();
            },
            1 => {
                exec_alias(&match_vec[0], args)?;
            },
            _ => {
                let selection = Select::new()
                    .default(0)
                    .items(&match_vec[..])
                    .interact_opt()
                    .unwrap();
                if let Some(selection) = selection {
                    exec_alias(&match_vec[selection], args)?;
                }
            }
        }
    }
    return Ok(());
}

/// Execute the given alias, using the given arguments
fn exec_alias(alias: &str, args: Vec<String>) -> io::Result<()> {
    let path = alias_path().join(alias);
    let al = Alias::read(&alias, &path)?;
    if al.must_confirm() {
        if Confirmation::new().default(false)
        .with_text(&format!("Execute alias \"{}\"?", alias)[..]).interact()? {
            al.execute(args);
        } else {
            exec_nothing();
        }
    } else {
        al.execute(args);
    }
    return Ok(());
}

/// If no alias can be executed, we execute an empty command instead.
fn exec_nothing() {
    println!(" ");
}

/// Create a new alias, and save it to file
fn add_alias(alias: &str, cmd: &str) -> io::Result<()> {
    if is_reserved_keyword(alias) {
        error(&format!("\"{}\" cannot be used as an alias name; it is a reserved keyword.", alias));
        return Err(io::Error::new(io::ErrorKind::Other, "Reserved keyword."));
    }

    let al = Alias::new(alias.clone(), cmd.clone(), "", false);
    let path = alias_path().join(alias);
    if path.exists() {
        if Confirmation::new().with_text("Overwrite existing alias?").interact()? {
            return al.write(&path);
        } else {
            return Ok(());
        }
    } else {
        return al.write(&path);
    }
}

/// Add/change the description of an existing alias, and save it to file
fn add_description(alias: &str, description: &str) -> io::Result<()> {
    let path = alias_path().join(alias);
    let al = Alias::read(&alias, &alias_path().join(alias))?;
    let new_al = al.update_description(description.clone());
    return new_al.write(&path);
}

/// Update whether a confirmation prompt should be shown for an existing alias, and save it to file
fn set_confirmation(alias: &str, confirm: bool) -> io::Result<()> {
    let path = alias_path().join(alias);
    let al = Alias::read(&alias, &path)?;
    let new_al = al.update_confirm(confirm);
    return new_al.write(&path);
}

/// Remove the file of an existing alias
fn remove_alias(alias: &str) -> io::Result<()> {
    let path = alias_path().join(alias);
    if path.exists() {
        return fs::remove_file(path);
    } else {
        error(&format!("there is no alias named \"{}\".", alias));
        return Err(io::Error::new(io::ErrorKind::NotFound, "Alias not found."));
    }
}