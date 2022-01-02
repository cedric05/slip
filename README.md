Simple `slip` command (git clone runner to better categorize personal repos and work repos).


example config file
```toml
# ~/.slip.toml
default = "Work"

[work]
# root directory to clone for work related projects
root = "/home/cedric05/projects/work"


[personal]
# root directory to clone for personal related projects
root = "/home/cedric05/projects/personal/"
```
## Install

`cargo install slip_git`

## example

`slip clone git@github.com/gitignore/gitgnore`
> with default configuration, it will create a repository in `/home/<username>/projects/work/<gitignore>/gitignore`


`slip -p clone git@github.com/microsoft/vscode`
> it will create a repository in `/home/<username>/projects/personal/<gitignore>/gitignore`

create file `~/.slip.toml` to better configure directories. 