use std::env;
use std::io;
extern crate dialoguer;
extern crate dirs;
use std::path::{Path, PathBuf};
mod alias;
use alias::Alias;
use dialoguer::{theme::ColorfulTheme, theme::CustomPromptCharacterTheme, Input};
use dialoguer::{Confirmation, Select};
use std::fs;

/// Jumpcut - a command-line utility to quickly access frequently-used commands/folders
/// Run without any parameters to display the usage message.
fn main() {
    // let selections = &[
    //     "Ice Cream",
    //     "Vanilla Cupcake",
    //     "Chocolate Muffin",
    //     "A Pile of sweet, sweet mustard",
    // ];

    // let selection = Select::new()
    //     .with_prompt("Pick your flavor")
    //     .default(0)
    //     .items(&selections[..])
    //     .interact_opt()
    //     .unwrap();
    // if let Some(selection) = selection {
    //     println!("Enjoy your {}!", selections[selection]);
    // } else {
    //     println!("You didn't select anything!");
    // }

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        usage();
        return;
    }

    match fs::create_dir_all(config_path()) {
        Ok(_v) => (),
        Err(_v) => error(&String::from("could not create configuration directory")),
    };

    let action = &args[1];
    match action.as_ref() {
        "is_exec_action" => {
            if args_ok(&args, 1) {
                println!("{}", !is_reserved_keyword(&args[2]));
            }
        }

        "list" => {
            match list_aliases() {
                Ok(_) => (),
                Err(e) => error(&e.to_string()),
            };
        }

        "add" => {
            if args_ok(&args, 2) {
                let cmd = args[3..].join(" ");
                add_alias(&args[2], &cmd);
            }
        }

        "addwd" => {
            if args_ok(&args, 2) {
                let abs_pwd = absolute_path(env::current_dir().unwrap());
                let cmd = args[3..].join(" ");
                add_alias(&args[2], &format!("cd {};{}", abs_pwd, cmd));
            }
        }

        "addpath" => {
            if args_ok(&args, 2) {
                let abs_path = absolute_path(PathBuf::from(args[3].clone()));
                add_alias(&args[2],
                        &format!("cd {}", abs_path),
                    );
            }
        }

        "desc" => {
            if args_ok(&args, 2) {
                let desc = args[3..].join(" ");
                add_description(&args[2], &desc);
            }
        }

        "rm" => remove_alias(&args[2]),

        _ => exec_alias(action, args[2..].to_vec()),
    };
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

fn exec_alias(alias: &String, args: Vec<String>) {
    let path = config_path().join(alias);
    match Alias::read(&alias, &path) {
        Ok(al) => al.execute(args),
        Err(e) => error(&e.to_string()),
    }
}

fn add_alias(alias: &String, cmd: &String) {
    if is_reserved_keyword(alias) {
        error(&format!(
            "\"{}\" cannot be used as an alias name; it is a reserved keyword.",
            alias
        ));
        return;
    }

    let al = Alias::new(alias.clone(), cmd.clone(), "".to_string());
    let path = config_path().join(alias);
    if path.exists() {
        match Confirmation::new()
            .with_text("Overwrite existing alias?")
            .interact()
        {
            Ok(_v) => {
                match al.write(&path) {
                    Ok(_) => (),
                    Err(_) => error(&format!("Could not save alias \"{}\".", alias)),
                };
            }
            Err(_v) => (),
        }
    } else {
        match al.write(&path) {
            Ok(_) => (),
            Err(_) => error(&format!("Could not save alias \"{}\".", alias)),
        };
    }
}

fn add_description(alias: &String, description: &String) {
    let path = config_path().join(alias);
    println!("{}", path.display());
    match Alias::read(&alias, &path) {
        Ok(al) => {
            let new_al = Alias::new(alias.clone(), al.get_command().clone(), description.clone());
            match new_al.write(&path) {
                Ok(_) => (),
                Err(e) => error(&e.to_string()),
            }
        }
        Err(e) => error(&e.to_string()),
    }
}

fn remove_alias(alias: &String) {
    let path = config_path().join(alias);
    if path.exists() {
        match fs::remove_file(path) {
            Ok(_v) => (),
            Err(_v) => error(&format!("Could not remove alias \"{}\".", alias)),
        }
    } else {
        error(&format!("There is no alias named \"{}\".", alias));
    }
}

/// Returns path to config directory
fn config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    return home.join(".jumpcut");
}

fn absolute_path(path: PathBuf) -> String {
    match fs::canonicalize(path) {
        Ok(v) => {
            let abs_path = v.into_os_string().into_string().unwrap();
            if abs_path.starts_with("\\\\?\\") {
                /* On Windows, Rust usually works with the "extended length path" / UNC path format, which has a \\?\  prefix.
                While perfectly fine, if you `cd` to such a path in Powershell, this absurdly long "Microsoft.PowerShell.Core\FileSystem::\\?\" prefix
                is shown in your shell. Because of this, I'm stripping the \\?\ prefix to convert it back to a normal path.
                The only caveat to normal paths is that they usually have a 260 max. character limit: 
                https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#maximum-path-length-limitation */
                return abs_path[4..].to_string();
            }
            return abs_path;
        },
        Err(_v) => {
            error(&String::from("the provided path does not exist"));
            panic!();
        }
    }
}

fn args_ok(args: &Vec<String>, num: usize) -> bool {
    if args.len() >= num + 2 {
        return true;
    } else {
        error(&format!("incorrect number of arguments; {} expected", num));
        return false;
    }
}

fn error(err: &String) {
    println!("Error: {}", err);
    usage();
}

/// Prints usage message
fn usage() {
    print!("
Jumpcut usage:

    j list                      List all aliases
    j [alias]                   Execute the alias named [alias] (also works by entering only part of its name)
    j [alias] [arg-1]..[arg-n]  Execute [alias], using the given arguments
    j add [alias] [cmd]         Adds a new alias, which executes the given command (arguments can be specified using $1, $2, ..)
    j addwd [alias] [cmd]       Adds a new alias, which always executes the given command from the current working directory
    j addpath [alias] [path]    Adds a new alias, which navigates to the given path
    j desc [alias] [desc]       Sets the description of [alias]
    j rm [alias]                Removes [alias]")
}
