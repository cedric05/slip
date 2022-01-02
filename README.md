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