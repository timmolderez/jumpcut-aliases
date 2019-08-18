# Jumpcut - terminal alias manager

Jumpcut is a utility for your terminal that lets you define aliases/shortcuts to quickly access folders or run commands. If you're familiar with the `alias` command, Jumpcut is similar, but provides some additional conveniences such as:
- Adding permanent aliases (without editing any .profile files)
- Executing an alias by only typing part of its name
- Quickly defining aliases that navigate to a given folder
- Defining parametrized aliases

Jumpcut is available for Bash (Linux/OS X) and Powershell (Windows), and can be easily ported to support other shells as well.

## Installation

1. Download the Jumpcut binary.
2. Modify your shell's .profile file. The exact instructions to do this depend on which shell you're using: Bash or Powershell. (If you're using another shell, it should be quite easy to add support.)

### Bash

TODO

```bash
function j {
  jumpcut_bin=/Users/soft/Documents/Git/jumpcut/target/debug/jumpcut
  
  if [ `$jumpcut_bin is_exec_action $1` == "true" ]; then
    cmd=`$jumpcut_bin $*`
    eval "$cmd"
  else
    eval "$jumpcut_bin $*"
  fi
}
```

### Powershell

TODO

```powershell
function j {
  $jumpcut_bin = 'C:\Users\Tim\Documents\Git\jumpcut\target\debug\jumpcut.exe'
  
  if ((Invoke-Expression "$jumpcut_bin is_exec_action $($args[0])") -eq "true") {
    $cmd = Invoke-Expression "$jumpcut_bin $args"
    Invoke-Expression $cmd
  } else {
    Invoke-Expression "$jumpcut_bin $args"
  }
}
```

### Other shells

TODO

## Usage

### Overview

`j`  - Print usage message

`j list` - Display the list of all aliases

`j [alias]` - Execute the alias named [alias] (also works by entering only part of its name)

`j [alias] [arg-1]..[arg-n]` - Execute [alias], using the given arguments

`j add [alias] [cmd]` - Adds a new alias, which executes the given command (arguments can be specified using $1, $2, ..)

`j addwd [alias] [cmd]` - Adds a new alias, which always executes the given command from the current working directory

`j addpath [alias] [path]` - Adds a new alias, which navigates to the given path

`j desc [alias] [desc]` - Sets the description of [alias]

`j rm [alias]` - Removes [alias]

### Adding aliases

TODO

### Executing aliases

TODO
