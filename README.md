# xdm
XXiaoA's dotfiles manager

## Install
Clone the source code with git. Then run `cargo r`

## Usage
> Run `xdm -h` for more details

First you should create a **yaml** file named `xdm.yaml` (not prerequisite, but **recommend**).

Then you can run `xdm` in a directory wich has the yaml file. Xdm will find the `xdm.yaml` automatically in the current directory. Or you're able to use `xdm file.yaml` to specify a yaml file.

## Example
```yaml
link:
  ./path-to-original-file:
    path: ./path-to-linked-file

  ./tmux:
    path: ~/.tmux.conf
```

| Parameter | Explanation                                      | type   | default |
| ---       | ---                                              | ---    | :---:   |
| path      | The file path to linked file                     | string | \\      |
| exist     | only create the link if the original file exists | bool   | true    |


## License
[GNU General Public License v3.0](./LICENSE)
