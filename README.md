Simple `slip` command (git clone runner to better categorize personal repos and work repos).


example config file
```toml
# ~/.slip.toml
default = "Work"

[work]
# root directory to clone for work related projects
root = "/home/cedric05/projects/work"
[work.git]
email = "some_email@company.com"
name = "name"


[personal]
# root directory to clone for personal related projects
root = "/home/cedric05/projects/personal/"
[personal.git]
email = "some_email@hotmail.com"
name = "name"

```
## Install

`cargo install slip_git`

## commands

### List
lists all cloned repos

example: `slip list`
### Reconfig
Reconfigures all git repos with correct email and name

example: `slip reconfig`

### Ui
Creats terminal `UI` (filters, select...) for opening in `vscode`

example: `slip ui`


### Clone

`slip clone git@github.com/gitignore/gitgnore`
> with default configuration, it will create a repository in `/home/<username>/projects/work/<gitignore>/gitignore`


`slip -p clone git@github.com/microsoft/vscode`
> it will create a repository in `/home/<username>/projects/personal/<gitignore>/gitignore`

create file `~/.slip.toml` to better configure directories. 