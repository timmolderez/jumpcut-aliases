use std::path::PathBuf;
use std::fs;
use std::ffi::OsStr;
use std::env;
use dialoguer::console::{style, Style};
use dialoguer::theme::ColorfulTheme;

pub const JUMPCUT_SHARED_ENV_VAR: &str = "JUMPCUT_SHARED_PATH";

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

/// Returns the shared alias storage path, if it has been configured
pub fn alias_shared_path() -> Option<PathBuf> {
    match env::var(JUMPCUT_SHARED_ENV_VAR) {
        Ok(x) => Some(PathBuf::from(x)),
        Err(_) => None
    }
}

/// Returns a theme that can be used in dialoguer's widgets
pub fn dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        active_item_prefix: style(">".to_string()).for_stderr(),
        ..ColorfulTheme::default()
    }
}

pub fn accent_style() -> Style {
    return Style::new().cyan();
}

/// Converts a `PathBuf` path to its absolute `String` representation
pub fn absolute_path(path: &PathBuf) -> String {
    match fs::canonicalize(path) {
        Ok(v) => {
            let abs_path = v.into_os_string().into_string().unwrap();
            return if abs_path.starts_with("\\\\?\\") {
                /* On Windows, Rust usually works with the "extended length path" / UNC path format, which has a \\?\  prefix.
                While perfectly fine, if you `cd` to such a path in Powershell, this absurdly long "Microsoft.PowerShell.Core\FileSystem::\\?\" prefix
                is shown in your shell. Because of this, I'm stripping the \\?\ prefix to convert it back to a normal path.
                The only caveat to normal paths is that they usually have a 260 max. character limit: 
                https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#maximum-path-length-limitation */
                abs_path[4..].to_string()
            } else {
                abs_path
            }
        },
        Err(_v) => {
            error("the provided path does not exist").ok();
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
    return if args.len() >= num + 2 {
        true
    } else {
        error(&format!("incorrect number of arguments; {} expected", num)).ok();
        usage();
        false
    }
}

/// Prints an error message to stderr
pub fn error(err: &str) -> std::io::Result<()> {
    eprintln!("Error: {}", err);
    // return Err(io::Error::new(io::ErrorKind::Other, err));
    return Ok(());
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
    j addshr ALIAS CMD            Identical to `j add`, but stores the alias in JUMPCUT_SHARED_PATH

    j desc ALIAS DESC             Sets the description of ALIAS to DESC
    j confirm ALIAS 0|1|2         Set alias confirmation prompt (0: none ; 1: y/n confirmation ; 2: explicit confirmation)
    j cp ALIAS1 ALIAS2            Copies ALIAS1 to ALIAS2, and optionally fill in any parameters
    j cpshr ALIAS1 ALIAS2         Identical to `j cp` , but stores the copied alias in JUMPCUT_SHARED_PATH
    j rm ALIAS                    Removes ALIAS

    Reference documentation: https://github.com/timmolderez/jumpcut-aliases/blob/master/README.md
    ")
}
