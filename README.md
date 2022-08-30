# xdm
XXiaoA's dotfiles manager

## Install
### Releases
Download the file from [releases](https://github.com/XXiaoA/xdm/releases)

### Crates.io
Download from [crates.io](https://crates.io/crates/xdm): `cargo install xdm`. And remember to add `~/.cargo/bin/` into your $PATH

### From source
Clone the source code with git. Then run `cargo install --path .`. And remember to add `~/.cargo/bin/` into your $PATH


## Usage
> Run `xdm -h` for more details

First you should create a **yaml** file named `xdm.yaml` (not prerequisite, but **recommend**).

Then you can run `xdm` in a directory which has the yaml file. Xdm will find the `xdm.yaml` automatically in the current directory. Or you're able to use `xdm file.yaml` to specify a yaml file.


## Configuration
For example:
```yaml
link:
  ./path-to-original-file:
    path: ./path-to-linked-file

  ./nvim: ~/repos/nvim

  ./tmux:
    path: ~/.tmux.conf
    if: test -e /usr/bin/tmux # for fish shell

create:
  - ~/repos
```
Notice: you must have `link` option.


###  Link
Link a file/directory.

What's more, the two following form is same, it can reduce your work: 
```yaml
link:
  ./a:
    path: b

  ./a: b
```

| Parameter | Explanation                                           | type   | default |
| ---       | ---                                                   | ---    | :---:   |
| path      | The file path to linked file                          | string | \\      |
| exist     | Only create the link if the original file exists      | bool   | true    |
| force     | Create the link whether the linked file exists or not | bool   | false   |
| if        | Create the link if shell command is true (WIP)        | string | \\      |
| create    | Create the parent directory of link if need           | bool    | true    |

### Create
Create a directory


## Todo:
can link file without configuration, for example:
```bash
$ xdm tmux # tmux is a directory or file
```
Then xdm will auto create the link


## License
[GNU General Public License v3.0](./LICENSE)
