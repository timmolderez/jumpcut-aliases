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

pub fn alias_shared_path() -> Option<PathBuf> {
    match env::var("JUMPCUT_SHARED_PATH") {
        Some(x) => Some(PathBuf::from(x)),
        None => None
    }
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

    j ALIAS                       Execute the alias called ALIAS (also works by only entering parts of its name)
    j list [SEARCH]               List all aliases (containing SEARCH in their name)
    j add ALIAS CMD               Adds a new alias, which executes the given command (parameter syntax: ?[PARAM])
    j addwd ALIAS CMD             Adds a new alias, which executes the given command, always from this working directory
    j addpath ALIAS [PATH]        Adds a new alias, which navigates to the given path (default path: \".\")
    j addshr ALIAS CMD            Identical to `j add`, but stores the alias in $JUMPCUT_SHARED
    j desc ALIAS DESC             Sets the description of ALIAS to DESC
    j confirm ALIAS 0|1|2         Set alias confirmation prompt (0: none ; 1: y/n confirmation ; 2: explicit confirmation)
    j cp ALIAS1 ALIAS2            Copies ALIAS1 to ALIAS2, and optionally fill in any parameters
    j rm ALIAS                    Removes ALIAS

    Reference documentation: https://github.com/timmolderez/jumpcut-aliases/blob/master/README.md
    ")
}
