extern crate dialoguer;
extern crate dirs;
extern crate regex;

use std::collections::HashMap;
use std::env;
use std::io;
use std::fs;
use std::path::{Path, PathBuf};
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
                return add_alias(&args[2], &cmd, &alias_path());
            }
        }

        "addwd" => {
            if args_ok(&args, 2) {
                let abs_pwd = absolute_path(&env::current_dir().unwrap());
                let cmd = args[3..].join(" ");
                return add_alias(&args[2], &format!("cd \"{}\";{};cd ?pwd", abs_pwd, cmd), &alias_path());
            }
        }

        "addpath" => {
            if args_ok(&args, 1) {
                let path = if args.len() > 3 {args[3..].join(" ")} else {".".to_string()};
                let abs_path = absolute_path(&PathBuf::from(path));
                return add_alias(&args[2], &format!("cd \"{}\"", abs_path), &alias_path());
            }
        }

        "addshr" => {
            if args_ok(&args, 2) {
                let cmd = args[3..].join(" ");
                return match alias_shared_path() {
                    Some(shared_path) => add_alias(&args[2], &cmd, &shared_path),
                    None => {
                        return error(&format!("no shared storage path configured! Please set the {} environment variable.", JUMPCUT_SHARED_ENV_VAR));
                    }
                };
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
                copy_alias(&args[2], &args[3], alias_path())?;
            }
        },

        "cpshr" => {
            if args_ok(&args, 2) {
                match alias_shared_path() {
                    Some(x) => copy_alias(&args[2], &args[3], x)?,
                    None => error(&format!("no shared storage path configured! Please set the {} environment variable.", JUMPCUT_SHARED_ENV_VAR))?
                }
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
            }
            find_and_exec_alias(alias_name_parts, alias_args).ok();
        }
    };

    return Ok(());
}

/// Is `action` a reserved keyword or is it an alias name?
fn is_reserved_keyword(action: &str) -> bool {
    let reserved_keywords = [
        "is_exec_action", "list",
        "add", "addwd", "addpath", "addshr",
        "desc", "confirm", "rm", "cp", "cpshr"];
    return reserved_keywords.contains(&action);
}

/// Finds all aliases that contain all given search strings
fn find_aliases(alias_parts: &Vec<String>, search_path: Option<PathBuf>) -> Vec<String> {
    if search_path.is_none() {
        return Vec::new();
    }

    let search_path_val = search_path.unwrap();
    let matches = search_path_val.read_dir().unwrap().filter_map(|entry| {
        let fname_str = osstr_to_string(entry.unwrap().file_name().as_os_str());

        let match_found = alias_parts.iter().all(|alias_part| {fname_str.contains(alias_part)});
        return if match_found {Some(fname_str)} else {None};
    });

    let mut match_vec: Vec<String> = matches.collect();
    match_vec.sort();
    return match_vec;
}

/// Given the name of an alias, determine where its alias file is stored, and read it
fn load_alias(alias: String) -> Option<Alias> {
    let in_default_path = alias_path().join(&alias).exists();
    let in_shared_path = match alias_shared_path() {
        Some(x) => x.join(&alias).exists(),
        None => false
    };

    if !in_default_path && !in_shared_path {
        return None;
    }

    let mut chosen_path = alias_path();
    if in_default_path && in_shared_path {
        let selection = Select::with_theme(&dialoguer_theme())
            .default(0)
            .items(&vec![&alias, &format!("{} (shared)", &alias)])
            .interact_opt()
            .unwrap().unwrap();
        if selection == 1 {
            chosen_path = alias_shared_path().unwrap();
        }
    } else if in_shared_path {
        chosen_path = alias_shared_path().unwrap();
    }

    return Some(Alias::read(&alias, &chosen_path.join(&alias)).unwrap());
}

/// Displays a list of all aliases, together with their command and description
fn list_aliases(alias_parts: Vec<String>) -> io::Result<()> {
    let matches = find_aliases(&alias_parts, Some(alias_path()));
    let matches_shared = find_aliases(&alias_parts, alias_shared_path());

    let all_matches = matches.iter().chain(matches_shared.iter());

    // Find the length of the longest alias; we need this for formatting the output
    let alias_len = all_matches.fold(0, |current_max, name| {
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

    if matches_shared.len() != 0 {
        println!("\nAliases in shared folder: ({})\n", alias_shared_path().unwrap_or_default().to_str().unwrap())
    }
    for entry in matches_shared {
        let path = alias_shared_path().unwrap().join(&entry);
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
            exec_alias(alias, args_map, alias_path())?;
            return Ok(());
        }
    }

    // If the user entered parts of an alias name
    let mut matches = find_aliases(&alias_parts, Some(alias_path()));
    let shared_matches = find_aliases(&alias_parts, alias_shared_path());
    let total_matches = matches.len()+shared_matches.len();

    match total_matches {
        0 => {
            exec_nothing();
            error("no matching aliases found.")?;
        },
        1 => {
            exec_alias(if matches.len() == 1 {&matches[0]} else {&shared_matches[0]} ,
                       args_map,
                       if shared_matches.len() > 0 {alias_shared_path().unwrap()} else {alias_path()})?;
        },
        _ => {
            // Multiple matches; ask the user to choose
            let shared_matches_suffixed = shared_matches.iter().map(|x| format!("{} (shared)", x));
            let matches_len = matches.len();
            matches.extend(shared_matches_suffixed);
            let selection = Select::with_theme(&dialoguer_theme())
                .default(0)
                .items(&matches[..])
                .interact_opt()
                .unwrap().unwrap();
            if selection >= matches_len {
                exec_alias(&shared_matches[selection-matches_len],
                           args_map,
                           alias_shared_path().unwrap())?;
            } else {
                exec_alias(&matches[selection], args_map, alias_path())?;
            }
        }
    }
    return Ok(());
}

/// If an expected argument value was not provided, prompt for it
fn missing_argument_handler(alias: &Alias, arg_name: &str) -> String {
    let input = Input::<String>::new().with_prompt(
        &format!("Enter a value for ?[{}] in `{}`", arg_name, &alias.get_command()[..])).interact().unwrap_or_default();
    return input
}

/// Execute the given alias, using the given arguments
fn exec_alias(alias: &str, args: HashMap<String, String>, storage_path: PathBuf) -> io::Result<()> {
    let path = storage_path.join(alias);
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
                error("unexpected input.")?;
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
fn add_alias(alias: &str, cmd: &str, storage_path: &Path) -> io::Result<()> {
    if is_reserved_keyword(alias) {
        return error(&format!("\"{}\" cannot be used as an alias name; it is a reserved keyword.", alias));
    }

    let alias_path = storage_path.join(alias);
    let al = Alias::new(alias.clone(), cmd.clone(), "", 0, alias_path.clone());

    if alias_path.exists() {
        if Confirm::new().with_prompt("Overwrite existing alias?").interact()? {
            return al.write(&alias_path);
        } else {
            return Ok(());
        }
    } else {
        return al.write(&alias_path);
    }
}

fn copy_alias(alias_source: &str, alias_target: &str, target_folder: PathBuf) -> io::Result<()> {
    return match load_alias(alias_source.to_string()) {
        Some(al) => {
            let copied_command = al.fill_in_parameters(
                al.get_command().to_string(), HashMap::new(),
                &fill_in_argument_handler, false);
            add_alias(alias_target, &copied_command, &target_folder).ok();
            add_description(alias_target, al.get_description()).ok();
            set_confirmation(alias_target, al.get_confirmation_level()).ok();
            return Ok(());
        },
        None => error(&format!("alias {:?} does not exist.", alias_source))
    };
}

/// When copying an alias, the user can choose to fill in an argument, or not
fn fill_in_argument_handler(alias: &Alias, arg_name: &str) -> String {
    let input = Input::<String>::new().with_prompt(
        &format!("Enter a value for ?[{}] in `{}` (or leave empty to keep it as an argument) ", arg_name, &alias.get_command()[..])).default("".to_string()).interact().unwrap_or_default();
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
    return match load_alias(alias.to_string()) {
        Some(x) => {
            let fp = &x.get_file_path().clone();
            return modify_fn(x).write(&fp);
        },
        None => error(&format!("alias {:?} does not exist.", alias))
    };
}

/// Remove the file of an existing alias
fn remove_alias(alias: &str) -> io::Result<()> {
    return match load_alias(alias.to_string()) {
        Some(x) => {
            fs::remove_file(x.get_file_path())},
        None => error(&format!("alias {:?} does not exist.", alias))
    };
}
