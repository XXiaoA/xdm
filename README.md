# xdm
XXiaoA's dotfiles manager

## Install
Clone the source code with git. Then run `cargo r`

## Usage
> Run `xdm -h` for more details

First you should create a **yaml** file named `xdm.yaml` (not prerequisite, but **recommend**).

Then you can run `xdm` in a directory wich has the yaml file. Xdm will find the `xdm.yaml` automatically in the current directory. Or you're able to use `xdm file.yaml` to specify a yaml file.

## Configurtion
```yaml
link:
  ./path-to-original-file:
    path: ./path-to-linked-file

  ./tmux:
    path: ~/.tmux.conf
    if: test -e /usr/bin/tmux # for fish shell

create:
  - ~/repos
```
###  Link
Link a file/directory
| Parameter | Explanation                                           | type   | default |
| ---       | ---                                                   | ---    | :---:   |
| path      | The file path to linked file                          | string | \\      |
| exist     | Only create the link if the original file exists      | bool   | true    |
| force     | Create the link whether the linked file exists or not | bool   | false   |
| if        | Create the link if shell command is true (WIP)        | string | \\      |
| create    | Create the parent directory of link if need           | bool    | true    |

### Create
Create a directory

## License
[GNU General Public License v3.0](./LICENSE)
