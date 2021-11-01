extern crate dialoguer;
extern crate dirs;
extern crate regex;

use std::collections::HashMap;
use std::env;
use std::io;
use std::fs;
use std::path::PathBuf;
use dialoguer::{Confirm, Select, Input};
use regex::Regex;

mod utils;
use utils::*;
pub mod alias;
use alias::Alias;


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
            if args.len()>2 {
                println!("{}", !is_reserved_keyword(&args[2]));
            } else {
                println!("false");
            }
        }

        "list" => {
            return list_aliases(args[2..].to_vec());
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
                return add_alias(&args[2], &format!("cd \"{}\";{};cd ?pwd", abs_pwd, cmd));
            }
        }

        "addpath" => {
            if args_ok(&args, 1) {
                let path = if args.len() > 3 {args[3..].join(" ")} else {".".to_string()};
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
                return set_confirmation(&args[2], args[3].parse::<i8>().unwrap_or_default());
            }
        }

        "cp" => {
            if args_ok(&args, 2) {
                copy_alias(&args[2], &args[3])?;
            }
        },

        "rm" => {
            if args_ok(&args, 1) {
                remove_alias(&args[2])?;
            }
        },

        _ => {
            let arg_regex = Regex::new(r"--([A-Za-z0-9_]+)=(.+)").unwrap();
            let mut alias_args = HashMap::new();
            let mut alias_name_parts = Vec::new();
            for arg in args[1..].iter() {
                match arg_regex.captures(arg) {
                    Some(x) => {alias_args.insert(x[1].to_string(), x[2].to_string()); ()},
                    None    => alias_name_parts.push(arg.clone())
                }


                // let mut match_found = false;
                // for cap in arg_regex.captures_iter(arg) {
                //     alias_args.insert(&cap[1], &cap[2]);
                //     let bla = &cap[0];
                //     match_found = true;
                // }
                // if !match_found  {
                //     alias_name_parts.push(arg);
                // }


                // let captures = arg_regex.captures(arg).unwrap();
                // if captures.len() == 0 {
                //     alias_name_parts.push(arg);
                // } else {
                //     let capture = captures[0];
                //     alias_args.insert(&captures[0][1], &captures[0][2]);
                // }
            }
            // let mut alias_name_parts = args[1..].iter().filter(
            //     |x| {
            //
            //     return true;}
            // );

            find_and_exec_alias(alias_name_parts, alias_args).ok();

            // let arg_split_index = args.iter().position(|x| x == "---").unwrap_or_default();
            //
            // return if arg_split_index == 0 {
            //     find_and_exec_alias(args[1..].to_vec(), Vec::new())
            // } else {
            //     find_and_exec_alias(args[1..arg_split_index].to_vec(), args[arg_split_index + 1..].to_vec())
            // }
        }
    };

    return Ok(());
}

/// Is `action` a reserved keyword or is it an alias name?
fn is_reserved_keyword(action: &str) -> bool {
    let reserved_keywords = [
        "is_exec_action", "list",
        "add", "addwd", "addpath", "rm",
        "desc", "confirm", "cp"];
    return reserved_keywords.contains(&action);
}

/// Finds all aliases that contain all given search strings
fn find_aliases(alias_parts: Vec<String>) -> Vec<String> {
    let matches = alias_path().read_dir().unwrap().filter_map(|entry| {
        let fname_str = osstr_to_string(entry.unwrap().file_name().as_os_str());

        let match_found = alias_parts.iter().all(|alias_part| {fname_str.contains(alias_part)});
        return if match_found {Some(fname_str)} else {None};
    });
    
    let mut match_vec: Vec<String> = matches.collect();
    match_vec.sort();
    return match_vec;
}

/// Displays a list of all aliases, together with their command and description
fn list_aliases(alias_parts: Vec<String>) -> io::Result<()> {
    let matches = find_aliases(alias_parts);

    // Find the length of the longest alias; we need this for formatting the output
    let alias_len = matches.iter().fold(0, |current_max, name| {
        return if name.len() > current_max {
            name.len()
        } else {
            current_max
        } 
    });

    for entry in matches {
        let path = alias_path().join(&entry);
        let al = Alias::read(&entry, &path)?;
        println!("{}", al.to_string(alias_len));
    }
    return Ok(());
}

/// Given (part) of an alias name, find any matches and execute it
/// 
/// If there are multiple matches, ask the user to choose one.
fn find_and_exec_alias(alias_parts: Vec<String>, args_map: HashMap<String, String>) -> io::Result<()> {
    // If the user entered a full alias name
    if alias_parts.len()==1 {
        let alias = &alias_parts[0];
        let path = alias_path().join(alias);
        if path.exists() {
            exec_alias(alias, args_map)?;
            return Ok(());
        }
    }

    // If the user entered parts of an alias name
    let matches = find_aliases(alias_parts);
    match matches.len() {
        0 => {
            error("no matching aliases found.");
            exec_nothing();
        },
        1 => {
            exec_alias(&matches[0], args_map)?;
        },
        _ => {
            let selection = Select::new()
                .default(0)
                .items(&matches[..])
                .interact_opt()
                .unwrap();
            if let Some(selection) = selection {
                exec_alias(&matches[selection], args_map)?;
            }
        }
    }
    return Ok(());
}

/// If an expected argument value was not provided, prompt for it
fn missing_argument_handler(alias: &Alias, arg_name: &str) -> String {
    let input = Input::<String>::new().with_prompt(
        &format!("Enter a value for ?[{}] in `{}`", arg_name, alias.get_command())[..]).interact().unwrap_or_default();
    return input
}

/// Execute the given alias, using the given arguments
fn exec_alias(alias: &str, args: HashMap<String, String>) -> io::Result<()> {
    let path = alias_path().join(alias);
    let al = Alias::read(&alias, &path)?;
    match al.get_confirmation_level(){
        0 => {al.execute(args, &missing_argument_handler)},
        1 => {
            if Confirm::new().default(false).with_prompt(&format!("Execute alias \"{}\"?", alias)[..]).interact()? {
                al.execute(args, &missing_argument_handler);
            } else {
                exec_nothing();
            }
        },
        2 => {
            let input = Input::<String>::new().with_prompt(&format!("Type \"{}\" to confirm", alias)[..]).interact()?;
            if input==alias {
                al.execute(args, &missing_argument_handler);
            } else {
                error("unexpected input.");
                exec_nothing();
            }
        }
        _ => {

        }
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

    let al = Alias::new(alias.clone(), cmd.clone(), "", 0);
    let path = alias_path().join(alias);
    if path.exists() {
        if Confirm::new().with_prompt("Overwrite existing alias?").interact()? {
            return al.write(&path);
        } else {
            return Ok(());
        }
    } else {
        return al.write(&path);
    }
}

fn copy_alias(alias_source: &str, alias_target: &str) -> io::Result<()> {
    let path = alias_path().join(alias_source);
    if path.exists() {
        let al = Alias::read(&alias_source, &path)?;
        let copied_command = al.instantiate_arguments(
            al.get_command().to_string(), HashMap::new(),
            &fill_in_argument_handler, false);
        add_alias(alias_target, &copied_command).ok();
        add_description(alias_target, al.get_description()).ok();
        set_confirmation(alias_target, al.get_confirmation_level()).ok();
        return Ok(());
    } else {
        error(&format!("There is no alias called \"{}\".", alias_source));
        return Err(io::Error::new(io::ErrorKind::Other, "Alias does not exist."));
    }
}

/// When copying an alias, the user can choose to fill in an argument, or not
fn fill_in_argument_handler(alias: &Alias, arg_name: &str) -> String {
    let input = Input::<String>::new().with_prompt(
        &format!("Enter a value for ?[{}] in `{}` (or leave empty to keep it as an argument)",
                 arg_name, alias.get_command())[..]).default("".to_string()).interact().unwrap_or_default();
    return input
}

/// Add/change the description of an existing alias, and save it to file
fn add_description(alias: &str, description: &str) -> io::Result<()> {
    return modify_alias(alias, |al|{
        return al.update_description(description.clone());
        });
}

/// Update whether a confirmation prompt should be shown for an existing alias, and save it to file
fn set_confirmation(alias: &str, confirm: i8) -> io::Result<()> {
    return modify_alias(alias, |al|{
        return al.update_confirm(confirm);
        });
}

/// Read an existing alias file, apply a modification function to it, and store the changes
fn modify_alias(alias: &str, modify_fn:impl Fn(Alias) -> Alias) -> io::Result<()> {
    let path = alias_path().join(alias);
    if !path.exists() {
      error(&format!("alias {:?} does not exist.", alias));
      return Ok(());
    }

    let al = Alias::read(&alias, &path)?;
    let new_al = modify_fn(al);
    return new_al.write(&path);
}

/// Remove the file of an existing alias
fn remove_alias(alias: &str) -> io::Result<()> {
    let path = alias_path().join(alias);
    return if path.exists() {
        fs::remove_file(path)
    } else {
        error(&format!("there is no alias named \"{}\".", alias));
        Ok(())
    }
}
