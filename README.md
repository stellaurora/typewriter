<h1 align="center">Typewriter</h1>

<div align="center">
🕰️☕🖋️📜💡
</div>
<div align="center">
  <strong>Configuration File Management Software</strong>
</div>
<div align="center">
  A <code>TOML</code> file-based configuration file management program for centralising configuration.
</div>


## 📜 Table of Contents
- [<code>📦 Getting Started</code>](#getting-started)
- [<code>🛠️ Features</code>](#features)
- [<code>📝 Usage</code>](#usage)
- [<code>⚙️ Config Examples</code>](#config-examples)
- [<code>📖 Configuration</code>](#configuration)
- [<code>🧾 License</code>](#license)
- [<code>🎓 Acknowledgments</code>](#acknowledgments)

<a name="getting-started"></a>
## 📦 Getting Started 

### For Linux x86_64

You can download prebuilt binaries off the latest runner build at **https://github.com/stellaurora/typewriter/releases/latest**. Then verify the checksum and simply provide execution permissions to the binary and use. The tiny version is compiled for binary size and uses the ``UPX`` on the binary for even more compression.

The entirety of typewriter can be used through this one binary, run it to view the initial ``help`` command to learn more or ``init`` to generate a basic template configuration file.

### For ???

Typewriter is for ``Linux`` and currently for github releases is only compiled under the CI for ``x86_64`` if you are running on another platform, you will need to manually clone and build typewriter yourself (unknown if it will work on other platforms!).

<a name="features"></a>
## 🛠️ Features

- Entirely ``TOML`` file-based configuration 
  - Supports "linking" (including) multiple ``TOML`` files together for a more modular configuration.

- Check-First Design
  - Checks for file permissions & errors first before making any real changes to the system.

- Variables
  - Command, Environment & Literal variables for usage in configuration files managed under typewriter.

- Checksum Checking
  - Checksum of files from prior ``apply`` commands is checked first on any subsequent apply for files, warning user's about the destination managed file having been changed outside of the typewriter system.

- Fault-Tolerant
  - Files that are being modified/changed have a backup placed in a temporary directory first just in case.

- Entirely Configurable
  - All of the above features can be totally/partially customised in functionality, If you don't want to check checksums then you don't need to!

<a name="usage"></a>
## 📝 Usage

The primary/entire use of typewriter is through the command:

```
typewriter apply --file <ROOT_CONFIG>
```

This will "apply" the files managed under typewriter to their destination locations, replacing them, A default template/configuration file for typewriter is provided and can be retrieved by running:

```
typewriter init --file <FILE_PATH>
```

The file argument is optional, and will simply default to ``typewriter.toml`` if not provided, The general flow of typewriter is to then edit this file (and associated ones) and use it with the ``apply`` command.

For any more information about the typewriter commands, the command:

```
typewriter help
```

can be used, though this does not cover the configuration options.


<a name="config-examples"></a>
## ⚙️ Config Examples

### Example/Default Configuration

This is a super minimal configuration that demonstrates the basic functionality of ``typewriter``, Read the comments in the file to learn more and read more about the various configuration options below..

```toml
# My First Typewriter Config!
#
# All file paths in typewriter are relative to the configuration file that they are contained in
# other than file paths potentially contained in variables, of which the CWD is the root configuration
# file.

# This indicates an actual file to be managed under typewriter
[[file]]

# This is the source of the file, or where the actual content
# should be sourced from to write to the destination
file="a.file"

# This is the location of which the source file will overwrite
# Typewriter does *not* create files if they do not already exist.
destination="~/.config/program/a.file"

# Another file to be managed.
[[file]]
file="b.file"
destination="~/.config/program/b.file"
```

### Links

Mechanism used for having multiple configuration files part of one typewriter config for more modularity.

```toml
# some_dir/typewriter_a.toml
#
# Including/having multiple files in one typewriter configuration
# Recursive/Multiple links to the same file are permitted but 
# each configuration file will only be used once regardless.

# Indicate a link to another file, like file
# multiple links can be provided in the same syntax
[[link]]
file = "subdir/typewriter_b.toml"
```

```toml
# some_dir/subdir/typewriter_b.toml
#
# Link to some other file!, remember
# file paths are relative to the current
# configuration file in [[link]] and [[file]].

[[link]]
file = "../typewriter_a.toml"
```

### Global Configuration

Want to modify the functionality of typewriter? This can be done through the ``config`` table, only the root configuration file ``config`` table will be used through, in order to remove any potential confusion (will not error though, only warn about unused config).

```toml
# Modify some configuration options!
# There are two main tables under config
#
# "apply" > apply command specific things
# "variables" > things specific to variable handling


[config.apply]
# Disable cleaning up temporary files from the end of apply
cleanup_files=false

# Change temporary/metadata file directory
apply_metadata_dir=".myconfigmetadata"
```

### Variables

Want to insert data from your central configuration into different configuration files? The ``variables`` feature of typewriter enables this. By default variables are referenced in configuration files by ``$TYPEWRITER{<variable_name>}`` but this can be customised.

```toml
# Variables example
# They can be in any config even linked ones, 
# but a variable can only be initialised one
# across all referenced configs to remove confusion.
#
# Undefined variables referenced with the same
# variable syntax will also error.

# A new variable
[[var]]

# Can be any characters other than whitespace, this is the
# variable name used by files
name="my_var"

# By default is literal (no need to provide), but can also be "command" and "environment"
type="literal"

# The literal value to replace the variable then
value="hello world!"

# Variables will be applied to all files
[[file]]
file="source.file"
destination="~/.config/source.file"
```

With the source file

```
print("$TYPEWRITER{my_var}")
```

Then, after we run ``apply`` the destination will be

```
print("hello world!")
```

<a name="configuration"></a>
## 📖 Configuration

These are the various configuration options available in typewriter configs.

### Global Configuration

These configuration options will only be taken in from the root configuration file referenced by the ``apply`` command, any global config referenced in ``link`` will be ignored in order to reduce confusion about configuration.

This is supplied under the ``[config]`` table of the file.

#### Apply

These can be referenced under the table ``[config.apply]`` in the toml and generally impact the ``apply`` command in some way.

------------------

##### ``auto_skip_unable_apply`` 

Whether or not to automatically skip files which do not meet the initial permission check

type: ``boolean``

```toml 
[conifg.apply]
auto_skip_unable_apply=false
```
------------------

##### ``confirm_apply``

Whether or not to skip the confirmation
prompt for the apply command

type: ``boolean``

```toml 
[conifg.apply]
confirm_apply=true
```

------------------

##### ``apply_metadata_dir``

Directory to place metadata/temporary files in
for the apply command

type: ``string``

```toml 
[conifg.apply]
apply_metadata_dir=".typewriter"
```


------------------

##### ``temp_copy_strategy``

Strategy for temporary copying functionality
for backup if failure occurs while applying

type: ``string``

**Valid Options:**


``copy_current``: Copy the current working file to the temporary directory while proceeding through the operation

``disabled``: Do not do any temporary copying
   

```toml 
[conifg.apply]
temp_copy_strategy="copy_current"
```

------------------

##### ``temp_copy_path_delim``

Delimiter for temp copy file names
to replace path seperator with, in file paths such as ``/home/me/this.file`` with a seperator of ``-`` this will become ``-home-me-this.file`` under the temporary directory.

type: ``string``

```toml 
[conifg.apply]
temp_copy_path_delim="-"
```

------------------

##### ``cleanup_files``

Should we clean up temporary copy files at the end of the apply if it succeeded?

type: ``bool``

```toml 
[conifg.apply]
cleanup_files=true
```

------------------

##### ``checkdiff_file_name``

Name of the checkdiff storage file for checkdiff in the metadata directory, this will store checksums/such data to check for later if the file was modified outside of typewriter.

type: ``string``

```toml 
[conifg.apply]
checkdiff_file_name=".checkdiff"
```

------------------

##### ``checkdiff_strategy``

Strategy of the checkdiff for checking if the file was modified out of the system just-in-case to not overwrite potential wanted files. Typewriter will prompt the user if the file was changed outside of the system (if this is not set to ``disabled``) before overwriting.

type: ``string``

**Valid Options:**


``xxhash``: Use xxhash to compute the checksum of files for comparison

``disabled``: Do not care if the files have been modified
   

```toml 
[conifg.apply]
checkdiff_strategy="xxhash"
```

------------------

##### ``checkdiff_skip_same``

Global toggle for whether checkdiff should be permitted to skip files if the content is found to be the same in source & destination or not, there is also a per-file granular one
which requires this to be set to true.

type: ``bool``

```toml 
[conifg.apply]
checkdiff_skip_same=true
```

------------------

##### ``skip_checkdiff_new``

Skip the checkdiff confirmation prompt if the entry is new to the checkdiff file and the checkdiff file was already initialised.

This occurs if the file has not been applied by typewriter yet.

type: ``bool``

```toml 
[conifg.apply]
skip_checkdiff_new=false
```

#### Variables

These can be referenced under the table ``[config.variables]`` in the toml and generally impact the handling/processing of variables in some way.



------------------

##### ``variable_format``

Variable format string to look for in the files to replace entirely with the variable value

**Format specifiers:**

``{variable}`` - Name of the variable being replaced

type: ``string``

```toml 
[conifg.variables]
variable_format="$TYPEWRITER{{variable}}"
```

------------------

##### ``variable_strategy``

Strategy to use for variable pre processing/whether to enable or disable variables.

type: ``string``

**Valid Options:**


``replace_variables``: Enabled, will preprocess and replace variables found in file using ``variable_format``

``disabled``: Do not preprocess files with variables/do not use variables.
   

```toml 
[conifg.variables]
variable_strategy="replace_variables"
```

------------------

##### ``variable_shell``

For ``command`` type variables, typewriter spawns the shell provided in this config option with the argument of ``shell_command_arg`` and runs the command variable using it, storing the standard output as the true value of the variable. This configuration option determines the executable to use for the shell.

type: ``string``

```toml 
[conifg.variables]
variable_shell="bash"
```

-----------------

##### ``shell_command_arg``

Argument to provide to the shell to be capable of running the variable command, for ``bash`` this is ``-c`` e.g. this will be provided as the first argument to the shell, and the command as the second argument.

type: ``string``

```toml 
[conifg.variables]
shell_command_arg="-c"
```

-----------------

##### ``confirm_shell_commands``

Confirm/prompt the user before running any shell commands per command.

type: ``bool``

```toml 
[conifg.variables]
confirm_shell_commands=true
```

### Links

This is an array of files specified each individually under the array table ``[[link]]``, each link is like including the file and will execute its contents as part of the typewriter system (excluding ``config`` for non-root configs).

---------------

#### ``file``

Links to another typewriter configuration file under the path supplied, this is relative to the current typewriter configuration file.

Can be used in any typewriter configuration file to "include" it into the overall configuration in order to have better modularity/cleaner file structure for the system configuration.

type: ``string``

```toml 
[[link]]
file="other_dir/other_typewriter_config.toml"
```

### Variables

These add individual "variables" (strings) that replace certain patterns supplied by ``variable_format`` in the configuration files managed by typewriter, each variable can be added under the array table ``[var]``

---------------

#### ``name``

Name of this variable, this should be unique and have no whitespace inbetween along with not being empty. Non-unique variables in a system will cause Error and abort the operation. Undeclared variables found in files will also result in the system aborting.

type: ``string``

```toml
[[var]]
name="my_variable"
```

------------------

#### ``type``

Type of this variable, which should be reflected in the value. This essentially determines where to pull the variable from. This is optional for variables, it does not need to be supplied and defaults to ``literal``

type: ``string``

**Valid Options:**


``literal``: Directly insert the value as a string in all references to the variable.

``command``: Execute the value as a command before preprocess occurs for all files, and insert its output in the references to the variable.

``environment``: Read in the value as an environment variable and insert the environment variables value in all references to the variable.

```toml 
[[var]]
type="literal"
```

------------------

#### ``value``

This links heavily into ``type``, but generally is what determines the content that the variables in config files will be replaced with.

type: ``string``

```toml 
[[var]]
value="hello world!"
```

### Files

These reference two files, the source and the destination for which to read files from and to overwrite, ``typewriter`` does not create files and will error/prompt to skip if they dont already exist!.

This is the core mechanism behind managing configuration and each file can be declared under the array table ``[[file]]``


------------------

### ``file``

The path to the source file, paths are relative to the current configuration file the ``[[file]]`` is in, using ``~`` or ``../`` e.g is permitted and should properly resolve. This is where the content will be pulled from into the destination.

type: ``string``

```toml 
[[file]]
file="source.file"
```

------------------

### ``skip_if_same_content``

Allow checkdiff to skip this file if the source file content is found to be the same as the destination content?

type: ``bool``

```toml 
[[file]]
skip_if_same_content=true
```

------------------

### ``destination``

The path to the destination file which will be overwritten by the source file on each apply. paths are relative to the current configuration file the ``[[file]]`` is in, using ``~`` or ``../`` e.g is permitted and should properly resolve.

type: ``string``

```toml 
[[file]]
destination="~/.config/destination.file"
```

<a name="license"></a>
## 🧾 License

This repository/``typewriter`` is under the MIT license. view it in the ``LICENSE`` file in the root directory of the repository.

<a name="acknowledgements"></a>
## 🎓 Acknowledgments

- Thanks to all of the crates used by ``typewriter``.
- Thanks to ``chezmoi`` & ``NixOS`` for inspiration.
- Thank you for reading this README/Learning about typewriter! ❤️

<br>

-------------

[Typewriter Authored/Created by Stellaurora](https://github.com/stellaurora/typewriter)

Love for everyone 💛 
