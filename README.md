# Jumpcut - alias manager

<img src="https://raw.githubusercontent.com/timmolderez/jumpcut/master/jumpcut.png" alt="Jumpcut logo">

Jumpcut is a utility for your terminal that lets you define aliases/shortcuts to quickly access folders or run commands. If you're familiar with the `alias` command, Jumpcut is similar, but provides some additional conveniences such as:
- Adding permanent aliases (without editing any .profile files)
- Executing an alias by only typing part of its name
- Quickly defining aliases that navigate to a given folder
- Defining parametrized aliases

Jumpcut is available for Bash (Linux/OS X) and Powershell (Windows), and can be easily ported to support other shells.

- [Jumpcut - alias manager](#jumpcut---alias-manager)
  - [Installation](#installation)
    - [Bash (Linux / Mac OS X)](#bash-linux--mac-os-x)
    - [Powershell (Windows)](#powershell-windows)
  - [Usage](#usage)
    - [Overview](#overview)
    - [Adding aliases](#adding-aliases)
    - [Executing aliases](#executing-aliases)
    - [Tips](#tips)
  - [Development](#development)

## Installation

### Bash (Linux / Mac OS X)

- [Download](http://timmolderez.be/builds/jumpcut/) the Jumpcut binary. (If there is no release for your platform, you can also [compile](#development) Jumpcut.)
- Open your profile script file. On Linux, your Bash profile script should normally be `~/.bashrc`. On Mac OS X, it should be `~/.bash_profile`. 
- Once opened, add the following snippet of code at the end:

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
- This snippet defines the `j` Bash function that is used to invoke Jumpcut. Make sure to adjust the `jumpcut_bin=~/jumpcut` line so it points to the path where you downloaded the Jumpcut binary! 
- Save the file.
- All done! The next time you open up a terminal, Jumpcut will be ready for use.

### Powershell (Windows)

- [Download](http://timmolderez.be/builds/jumpcut/) the Jumpcut binary. (If there is no release for your platform, you can also [compile](#development) Jumpcut.)
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

- This snippet defines the `j` Powershell function that is used to invoke Jumpcut. Make sure to adjust the `$jumpcut_bin = 'C:\jumpcut.exe'` line so it points to the path where you downloaded the Jumpcut binary.
- Save the file.
- In most cases, Windows' default security policy does not allow executing any Powershell scripts, including the profile script. While a reasonable safety precaution for most users, we'll want to open this up a bit. You can change Windows' policy so it does allow scripts that are created locally, but scripts downloaded from the internet must be digitally signed: open a Powershell window as administrator, then run `Set-ExecutionPolicy RemoteSigned`.<br />([Set-ExecutionPolicy documentation](https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.security/set-executionpolicy?view=powershell-6))
- All done! The next time you open up a Powershell window, Jumpcut will be ready for use.

## Usage

### Overview

`j`  - Print usage message

`j [alias]` - Execute the alias named [alias] (also works by entering only part of its name)

`j [alias] [arg-1]..[arg-n]` - Execute [alias], using the given arguments

`j list` - List all aliases, including their description and command

`j list [search]` - List all aliases containing [search] in their name

`j add [alias] [cmd]` - Adds a new alias, which executes the given command (arguments can be specified using ?1, ?2, ..)

`j addwd [alias] [cmd]` - Adds a new alias, which always executes the given command from the current working directory

`j addpath [alias] [path]` - Adds a new alias, which navigates to the given path

`j desc [alias] [desc]` - Sets the description of [alias] to [desc]; the description is removed if [desc] is "-"

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

Aliases can also be parametrized using `?1`, `?2`, `?3`, etc.<br />
*Define an alias `tgz` that executes `tar -czvf compressed.tar.gz ?1`*
```bash
~> j add tgz tar -czvf compressed.tar.gz ?1
~> j tgz Downloads
~> ... The folder ~/Downloads is now being archived to compressed.tar.gz ...
```

*Define an alias `tgz` that executes `tar -czvf ?1 ?2`*
```bash
~> j add tgz tar -czvf ?1 ?2
~> j tgz downloads.tar.gz Downloads
~> ... The folder ~/Downloads is now being archived to downloads.tar.gz ...
```

#### `j addpath`

The `j addpath` command defines an alias that will navigate to a given folder:

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

This example is equivalent to `j add jc-pull cd "home/user/Documents/Git/Jumpcut;git pull;cd ?pwd"`. (Note that `?pwd` is filled in by Jumpcut with the current directory whenever the alias is invoked.)

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
gcomm    git commit -m ?1
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

### Tips

#### Grouping aliases

Once you start developing a larger collection of aliases, a simple way to organise them is to use prefixes in the alias name. 

For example, I have several aliases that navigate to different folders relevant for one organisation, and other aliases to navigate to folders for another organisation. You can distinguish between the two by adding e.g. an "org1-" prefix or an "org2"-prefix in the alias name.

Because you only need to enter part of the alias name to execute it, this has two benefits:
- You don't need to bother with typing the prefix if you know exactly which alias you want.
- If you want to see all aliases of the "org1-" group, you can simply type `j org1-`.

#### Invoking Jumpcut with another name than `j`

To change the name you use to invoke Jumpcut, change the `function j {` line in your profile script (the one you modified during [installation](#installation)). For example, change it to `function x {`. As soon as you open a new terminal window, Jumpcut will now be invoked with `x`.

#### Show the actual command when executing an alias

If you'd like see which command is actually executed when invoking an alias, you only need to add one line to your profile script (the one you modified during [installation](#installation)):

*Bash* - Add `echo "$cmd"` just before the `eval "$cmd"` line.

*Powershell* - Add `echo $cmd` just before the `Invoke-Expression $cmd` line.

#### Adding alias commands with reserved symbols

If the command you'd like to alias contains any symbols that are reserved by your shell, these symbols should be escaped:

*Bash* - Add `"` quotes around the entire command, and add the `\` escape character before all reserved symbols. For example, you can create an alias `test` for the command `echo foo;echo bar` as follows: 
```
j add test "echo foo\;echo bar"
```

*Powershell* - Add `'` quotes around the entire command, and add the `` ` `` (backtick) escape character before all reserved symbols. For example, you can create an alias `test` for the command `echo foo;echo bar` as follows: 
```
j add test 'echo foo`;echo bar'
```

#### Manual alias management 

If needed, you can also manually manage aliases. Your aliases are stored as text files in the `.jumpcut` folder of your home directory. The file format of an alias is very simple:
- The name of the file is the alias name. (The file does not have an extension!) 
- The file itself contains exactly one line, which is the command to be executed.
- Optionally, you can add a description for the alias on the second line.

## Development

Jumpcut can be compiled as follows:
1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone Jumpcut's repository: `git clone git@github.com:timmolderez/jumpcut.git`
3. *(Optional)* Run Jumpcut's test suite: `cargo test -- --test-threads=1`
4. Run `cargo build --release`
5. All done! You can find the compiled binary in the "target/release" subdirectory.

Note that the Jumpcut binary itself won't execute any aliases; it can only print the command to be executed to your console. The actual execution is done by the snippet of code you had to add to your shell's profile script during Jumpcut's [installation](#installation).
