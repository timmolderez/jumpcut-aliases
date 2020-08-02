use std::path::PathBuf;
use std::fs;
use std::ffi::OsStr;

#[cfg(debug_assertions)]
use std::env;

/// Returns the path where aliases are stored (release build)
#[cfg(not(debug_assertions))]
pub fn alias_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    return home.join(".jumpcut");
}

/// Returns the path where all aliases are stored (debug and test builds)
#[cfg(debug_assertions)]
pub fn alias_path() -> PathBuf {
    let pwd = env::current_dir().unwrap_or_default();
    return pwd.join(".jumpcut_test");
}

/// Converts a `PathBuf` path to its absolute `String` representation
pub fn absolute_path(path: &PathBuf) -> String {
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
            } else {
                return abs_path;
            }
        },
        Err(_v) => {
            error("the provided path does not exist");
            panic!();
        }
    }
}

/// Converts a OsStr to a String
pub fn osstr_to_string(ostr: &OsStr) -> String {
    return ostr.to_os_string().into_string().unwrap()
}

/// Check the length of the given argument list
/// 
/// If the length is not as expected, false is returned and an error is printed.
pub fn args_ok(args: &Vec<String>, num: usize) -> bool {
    if args.len() >= num + 2 {
        return true;
    } else {
        error(&format!("incorrect number of arguments; {} expected", num));
        usage();
        return false;
    }
}

/// Prints an error message to stderr
pub fn error(err: &str) {
    eprintln!("Error: {}", err);
}

/// Prints usage message
pub fn usage() {
    print!("
    Jumpcut usage:

    j [alias]                     Execute the alias named [alias] (also works by entering only part of its name)
    j [alias] [arg-1]..[arg-n]    Execute [alias], using the given arguments
    j list                        List all aliases
    j list [search]               List all aliases containing [search] in their name
    j add [alias] [cmd]           Adds a new alias, which executes the given command (arguments can be specified using ?1, ?2, ..)
    j addwd [alias] [cmd]         Adds a new alias, which always executes the given command from the current working directory
    j addpath [alias] [path]      Adds a new alias, which navigates to the given path
    j desc [alias] [desc]         Sets the description of [alias] to [desc]
    j confirm [alias] 0|1|2       Set alias confirmation prompt (0: none ; 1: y/n confirmation ; 2: explicit confirmation)
    j rm [alias]                  Removes [alias]
    ")
}
