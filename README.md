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

You can download prebuilt binaries off the latest runner build at **https://github.com/duplessisaurore/typewriter/releases/latest**. Then verify the checksum and simply provide execution permissions to the binary and use. The tiny version is compiled for binary size and uses the ``UPX`` on the binary for even more compression.

The entirety of typewriter can be used through this one binary, run it to view the initial ``help`` command to learn more or ``init`` to generate a basic template configuration file.

### For ???

Typewriter is for ``Linux`` and currently for github releases is only compiled under the CI for ``x86_64`` if you are running on another platform, you will need to manually clone and build typewriter yourself (unknown if it will work on other platforms!).

<a name="features"></a>
## 🛠️ Features

- Entirely ``TOML`` file-based configuration 
  - Supports "linking" (including) multiple ``TOML`` files together for a more modular configuration.

- Atomic & Transactional Apply
  - All validation and permission checks happen before any files are modified.
  - If any operation fails, all changes are automatically rolled back from backups.
  - Either the entire apply succeeds or none of it does, ensuring system consistency.

- Variables
  - Command, Environment & Literal variables for usage in configuration files managed under typewriter.

- Checksum Checking
  - Checksum of files from prior ``apply`` commands is checked first on any subsequent apply for files, warning user's about the destination managed file having been changed outside of the typewriter system.
  
- Hooks
  - Run shell commands at different stages of the apply process.
  - Supports global ``pre_apply`` and ``post_apply`` hooks.
  - Supports per-file ``pre_hook`` and ``post_hook`` commands.

- Fault-Tolerant
  - Files that are being modified/changed have a backup placed in a temporary directory first just in case during the process of the copying.
  - Automatic rollback and cleanup on failure.

- Entirely Configurable
  - All of the above features can be totally/partially customised in functionality, If you don't want to check checksums then you don't need to!

<a name="usage"></a>
## 📝 Usage

The primary/entire use of typewriter is through the command:

```
typewriter apply --file <ROOT_CONFIG> --section <SECTION_NAME>
```

This will "apply" the files managed under typewriter to their destination locations, replacing them. The `--section` argument specifies the [Quill](https://github.com/duplessisaurore/quill) scope to extract from the TOML files (defaulting to "typewriter").

A default template/configuration file for typewriter is provided and can be retrieved by running:

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
# other than file paths potentially contained in variables, of which the CWD is the parent directory
# of the configuration file.

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

Want to modify the functionality of typewriter? This can be done through the `config` table, only the root configuration file `config` table will be used through, in order to remove any potential confusion (will not error though, only warn about unused config).

```toml
# Modify some configuration options!
# There are four main tables under config
#
# "apply" > apply command specific things
# "variables" > things specific to variable handling
# "commands" > config for all shell command execution
# "hooks" > config for hook execution

[config.apply]
# Disable cleaning up temporary files from the end of apply
cleanup_files=false

# Change temporary/metadata file directory
apply_metadata_dir=".myconfigmetadata"

[config.commands]
# Don't ask for confirmation before running variables or hooks
confirm_shell_commands = false
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

# A variable from a command
[[var]]
name="my_command_var"
type="command"
# This command is executed using the shell defined in [config.commands]
value="echo 'I came from a command!'"

# Variables will be applied to all files
[[file]]
file="source.file"
destination="~/.config/source.file"
```

With the source file

```
print("$TYPEWRITER{my_var}")
print("$TYPEWRITER{my_command_var}")
```

Then, after we run ``apply`` the destination will be

```
print("hello world!")
print("I came from a command!")
```

### Hooks

Run custom commands during the ``apply`` process at different phases of the ``apply``.

```toml
# Hooks example
# They can be in any config even linked ones, 

# Configure global hook behavior
[config.hooks]
# Can be "abort" (default) or "continue"
# which will be the strategy to use when
# a hook fails
failure_strategy = "continue"

# Define a global hook
# This will run after all files are applied
[[hook]]
command = "echo 'Finished applying all files!'"
stage = "post_apply"
# This specific hook will abort the apply if it fails
continue_on_error = false

# Define file-specific hooks
[[file]]
file = "source.file"
destination = "~/.config/source.file"

# Hooks to run *before* this file is applied
pre_hook = [
    "echo 'About to apply source.file...'",
    "touch ~/.config/source.file.log"
]

# Hooks to run *after* this file is applied
post_hook = [
    "echo 'Finished applying source.file!' > ~/.config/source.file.log"
]

# If true, apply will continue even if this file's hooks fail
continue_on_hook_error = false
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

Whether or not to automatically skip files which do not meet the initial permission check.

type: ``boolean``

```toml 
[conifg.apply]
auto_skip_unable_apply=false
```
------------------

##### ``confirm_apply``

Whether or not to skip the confirmation
prompt for the apply command.

type: ``boolean``

```toml 
[conifg.apply]
confirm_apply=true
```

------------------

##### ``apply_metadata_dir``

Directory to place metadata/temporary files in
for the apply command.

type: ``string``

```toml 
[conifg.apply]
apply_metadata_dir=".typewriter"
```


------------------

##### ``temp_copy_strategy``

Strategy for temporary copying functionality
for backup if failure occurs while applying. Backups are created before any files are modified and used for automatic rollback on failure.

type: ``string``

**Valid Options:**


``copy_all``: Copy all destination files to the temporary directory for backup before proceeding with the operation (default)

``disabled``: Do not do any temporary copying (disables automatic rollback)
   

```toml 
[config.apply]
temp_copy_strategy="copy_all"
```

------------------

##### ``temp_copy_path_delim``

Delimiter for temp copy file names
to replace path seperator with, in file paths such as ``/home/me/this.file`` with a seperator of ``-`` this will become ``-home-me-this.file`` under the temporary directory.

type: ``string``

```toml 
[config.apply]
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

##### ``file_permission_strategy``

Strategy for checking file permissions and optionally creating missing destination files before the apply operation proceeds. This runs during the validation phase to ensure all files are accessible before any modifications are made.

type: ``string``

**Valid Options:**

``check_only``: Only validates that all source files are readable and all destination files are writable. Will abort if any permission issues are found.

``create_if_missing``: Like ``check_only``, but also creates destination files if they don't exist yet. Created files are tracked and automatically cleaned up if the apply operation fails.

``disabled``: Disables file permission checking entirely.

```toml
[config.apply]
file_permission_strategy="check_only"
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

Variable format string to look for in the files to replace entirely with the variable value.

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


``replace_variables``: Enabled, will preprocess and replace variables found in file using ``variable_format``.

``disabled``: Do not preprocess files with variables/do not use variables.
   

```toml 
[conifg.variables]
variable_strategy="replace_variables"
```

#### Commands

These can be referenced under the table ``[config.commands]`` in the toml and control the execution of all shell commands (for both "command" variables and hooks).

------------------

##### ``shell``

The shell executable to use for running commands, will be spawned as a subprocess for each command.

type: ``string``

```toml
[config.commands]
shell="bash"
```

------------------

##### ``shell_command_arg``

Argument to provide to the shell to be capable of running the command. For ``bash`` this is ``-c``.

type: ``string``

```toml
[config.commands]
shell_command_arg="-c"
```

------------------

##### ``confirm_shell_commands``

Confirm/prompt the user before running any shell commands (from variables or hooks).

type: ``bool``

```toml
[config.commands]
confirm_shell_commands=true
```

------------------

##### ``commands_inherit_stdin``

Should commands inherit the standard input of typewriter and take the standard input of the user?

type: ``bool``

```toml
[config.commands]
commands_inherit_stdin=true
```

------------------

##### ``commands_inherit_stdout``

Should commands inherit the standard output of typewriter and have their output displayed to the user?

type: ``bool``

```toml
[config.commands]
commands_inherit_stdout=true
```

------------------

##### ``commands_inherit_stderr``

Should commands inherit the stderr of typewriter and have their stderr output displayed to the user?

type: ``bool``

```toml
[config.commands]
commands_inherit_stderr=true
```

#### Hooks

These can be referenced under the table ``[config.hooks]`` in the toml and control the global behavior of hooks.

------------------

##### ``hooks_enabled``

Whether or not hooks should be enabled in typewriter.

type: ``bool``

```toml
[config.hooks]
hooks_enabled=true
```

------------------

##### ``failure_strategy``

The strategy to handle hook failure and determine if the ``apply`` should still continue.

type: ``string``

**Valid Options:**

``abort``: Stop the entire apply operation (default).

``continue``: Continue with the apply operation.

```toml
[config.hooks]
failure_strategy="abort"
```

### Links

This is an array of files specified each individually under the array table ``[[link]]``, each link is like including the file and will execute its contents as part of the typewriter system (excluding ``config`` for non-root configs).

#### Aliases
The ``[[link]]`` tables can also be defined under the aliases ``[[include]]``, ``[[use]]``, and ``[[import]]``.

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

These add individual "variables" (strings) that replace certain patterns supplied by ``variable_format`` in the configuration files managed by typewriter, each variable can be added under the array table ``[[var]]``.

#### Aliases
The ``[[var]]`` tables can also be defined under the aliases ``[[variable]]`` and ``[[define]]``.

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

``command``: Execute the value as a command in the shell defined in ``[config.commands]`` before preprocessing occurs for all files, and inserts its output in the references to the variable.

``environment``: Read in the value as an environment variable and insert the environment variables value in all references to the variable.

```toml
[[var]]
type="literal"
```

------------------

#### ``value``

This links heavily into ``type``, but generally is what determines the content that the variables in config files will be replaced with.

Can also refer to other variables (as long as it is not a cyclic dependency) using the same variable format specifier as would be used in files.

type: ``string``

```toml
[[var]]
value="hello world!"
```

### Hooks

These define global commands to be run at specific stages of the apply process. Each hook can be declared under the array table ``[[hook]]``.

#### Aliases

The array table ``[[hook]]`` can also be defined under the alias ``[[command]]``.

------------------

#### ``command``

The shell command to execute. This will be run using the shell defined in ``[config.commands]``.

type: ``string``

```toml
[[hook]]
command="echo 'Starting apply...'"
```

------------------

#### ``stage``

The stage at which to execute this global hook.

type: ``string``

**Valid Options:**

``pre_apply``: Run before any files are processed.

``post_apply``: Run after all files have been processed.

```toml
[[hook]]
stage="post_apply"
```

------------------

#### ``continue_on_error``

If set to ``true``, this hook failing will not stop the apply operation, overriding the global ``failure_strategy`` for this hook only.

type: ``bool``

```toml
[[hook]]
continue_on_error=true
```

### Files

These reference two files, the source and the destination for which to read files from and to overwrite, `typewriter` does not create files and will error/prompt to skip if they dont already exist!.

This is the core mechanism behind managing configuration and each file can be declared under the array table ``[[file]]``.

#### Aliases

The ``[[file]]`` tables can also be defined under the alias ``[[track]]``.

------------------

#### ``file``

The path to the source file, paths are relative to the current configuration file the ``[[file]]`` is in, using ``~`` or ``../`` e.g is permitted and should properly resolve. This is where the content will be pulled from into the destination.

type: ``string``

```toml
[[file]]
file="source.file"
```

------------------

#### ``skip_if_same_content``

Allow checkdiff to skip this file if the source file content is found to be the same as the destination content? (Requires ``checkdiff_skip_same`` to be ``true`` in global config).

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

------------------

#### ``pre_hook``

A list of shell commands to execute *before* this specific file is applied. If checkdiff or another strategy which causes files to be not-applied, then this will not run.

type: ``list of strings``

```toml
[[file]]
pre_hook = [
    "echo 'Updating config.json...'",
    "mkdir -p .config/app"
]
```

------------------

#### ``post_hook``

A list of shell commands to execute *after* this specific file is applied. If checkdiff or another strategy which causes files to be not-applied, then this will not run.

type: ``list of strings`` 

```toml
[[file]]
post_hook = [
    "echo '...done updating config.json'"
]
```

------------------

#### ``continue_on_hook_error``

If set to ``true``, the apply operation will continue even if one of this file's ``pre_hook`` or ``post_hook`` commands fail.

type: ``bool``

```toml
[[file]]
continue_on_hook_error = true
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

[Typewriter Authored/Created by Aurore du Plessis](https://github.com/duplessisaurore/typewriter)

Love for everyone 💛 
