# Jumpcut - terminal alias manager

Jumpcut is a utility for your terminal that lets you define aliases/shortcuts to quickly access folders or run commands. If you're familiar with the `alias` command, Jumpcut is similar, but provides some additional conveniences such as:
- Adding permanent aliases (without editing any .profile files)
- Executing an alias by only typing part of its name
- Quickly defining aliases that navigate to a given folder
- Defining parametrized aliases

Jumpcut is available for Bash (Linux/OS X) and Powershell (Windows), and can be easily ported to support other shells as well.

- [Installation](#installation)
  - [Bash](#bash)
  - [Powershell](#powershell)
- [Usage](#usage)
  - [Overview](#overview)
  - [Adding aliases](#adding-aliases)
  - [Executing aliases](#executing-aliases)
- [Development](#development)

## Installation

- [Download](http://timmolderez.be/builds/jumpcut/) the Jumpcut binary. (If there is no release for your platform, you can also [compile](#development) Jumpcut.)
- Modify your shell's "profile" file, the script that is executed every time you open a new terminal window. The exact instructions to do this depend on which shell you're using: [Bash](#bash) or [Powershell](#powershell):

### Bash

- Open your profile script file. On Linux, your Bash profile script should normally be `~/.bashrc`. On Mac OS X, it should be `~/.bash_profile`. 
- Once you've opened the profile script, add the following snippet of code at the end:

```bash
function j {
  jumpcut_bin=~/jumpcut
  
  if [ `$jumpcut_bin is_exec_action $1` == "true" ]; then
    cmd=`$jumpcut_bin $*`
    eval "$cmd"
  else
    eval "$jumpcut_bin $*"
  fi
}
```
- This snippet defines the `j` Bash function that is used to invoke Jumpcut. If you'd like to invoke Jumpcut with another name than `j`, simply change the function name.
- Make sure to adjust the `jumpcut_bin=~/jumpcut` line so it points to the path where you downloaded the Jumpcut binary!
- Save the file.
- All done! The next time you open up a terminal, Jumpcut will be ready for use.

### Powershell

- Open your Powershell profile script. You can find out its location by entering `$profile` in a shell.
- Once opened, add the following snippet of code at the end:

```powershell
function j {
  $jumpcut_bin = 'C:\jumpcut.exe'
  
  if ((Invoke-Expression "$jumpcut_bin is_exec_action $($args[0])") -eq "true") {
    $cmd = Invoke-Expression "$jumpcut_bin $args"
    Invoke-Expression $cmd
  } else {
    Invoke-Expression "$jumpcut_bin $args"
  }
}
```

- This snippet defines the `j` Powershell function that is used to invoke Jumpcut. If you'd like to invoke Jumpcut with another name than `j`, simply change the function name.
- Make sure to adjust the `$jumpcut_bin = 'C:\jumpcut.exe'` line so it points to the path where you downloaded the Jumpcut binary!
- Save the file.
- All done! The next time you open up a terminal, Jumpcut will be ready for use.

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

`j confirm [alias] true|false`  If true, a confirmation prompt is shown when executing [alias]

`j rm [alias]` - Removes [alias]

### Adding aliases

Aliases can be added using either `j add`, `j addpath` or `j addwd`. Each of these serve a different purpose:

#### `j add`

This is the most general-purpose means to define an alias. Here are a few examples:

*Define an alias called `up` that executes the command `cd ..`*
```bash
~/Documents/Git> j add up cd ..
~/Documents/Git> j up
~/Documents>
```

Aliases can also be parametrized using `$1`, `$2`, `$3`, etc.<br />
*Define an alias `tgz` that executes `tar -czvf compressed.tar.gz $1`*
```bash
~> j add tgz tar -czvf compressed.tar.gz $1
~> j tgz Downloads
~> ... The folder ~/Downloads is now being archived to compressed.tar.gz ...
```

*Define an alias `tgz` that executes `tar -czvf $1 $2`*
```bash
~> j add tgz tar -czvf $1 $2
~> j tgz downloads.tar.gz Downloads
~> ... The folder ~/Downloads is now being archived to downloads.tar.gz ...
```

#### `j addpath`

The `j addpath` command defines an alias that will navigate to the given folder:

*Define alias `q2` that navigates to folder `~/Documents/Git/Quake-2`*
```bash
~/Documents> j addpath q2 ./Git/Quake-2
~/Documents> j q2
~/Documents/Git/Quake-2>
```
This example is equivalent to `j add q2 cd "/home/user/Documents/Git/Quake-2"`. <br />As you can tell, `j addpath` is a bit more convenient as you only need to provide a relative path.

#### `j addwd`

The `j addwd` command (where "wd" stands for "working directory") is useful for those situations where a command must be executed from a specific directory. For instance, you'd like to pull the latest changes from a specific git project.

*Define an alias `jc-pull` that runs `git pull` in directory `~/Documents/Git/Jumpcut`*
```bash
~/Documents/Git/Jumpcut> j addwd jc-pull git pull
~/Documents/Git/Jumpcut> cd ~
~> j jc-pull
... Pulling the latest changes from ~/Documents/Git/Jumpcut ...
~>
```

This example is equivalent to `j add jc-pull cd "home/user/Documents/Git/Jumpcut;git pull;cd $pwd"`. (Note that `$pwd` is filled in by Jumpcut with the current directory whenever the alias is invoked.)

### Executing aliases

An alias can be executed by typing `j` + its name:
```bash
~/Documents> j addpath docs .
~/Documents> cd ..
~>j docs
~/Documents>
```
It also is sufficient to type in part of its name:
```bash
~> j doc
~/Documents>
```
```bash
~> j oc
~/Documents>
```
Now, let's say we've already defined a couple of aliases: (`j list` will show all aliases)
```bash
~> j list
gpush    git push
gcomm    git commit -m $1
gpull    git pull
```
By entering `j push`, it is clear we intend to run the `gpush` alias. However, in case of `j gp`, it is not clear whether `gpush` or `gpull` should be executed. If there is any ambiguity, Jumpcut will display a selection menu:
```bash
~> j gp
> gpush
  gpull
```
Finally, while most aliases may be harmless if you execute them by accident, you may also define a couple aliases where this is not the case. To avoid such accidents, you can add a confirmation prompt to specific aliases using `j confirm`:
```bash
~> j confirm gpush true
~> j gpush
Execute alias "home"? [y/N]
```

## Development

Jumpcut can be compiled as follows:
1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone Jumpcut's repository: `git clone git@github.com:timmolderez/jumpcut.git`
3. Run `cargo build`
4. All done! You can find the compiled binary in the "target/debug" subdirectory.

Note that the Jumpcut binary itself won't execute any aliases; it can only print the command to be executed to your console. The actual execution is done by the snippet of code you had to add to your shell's profile script during Jumpcut's [installation](#installation).