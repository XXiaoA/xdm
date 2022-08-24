# xdm
XXiaoA's dotfiles manager

## Usage
> Run `xdm -h` for details

```yaml
link:
  ./path-to-original-file:
    path: ./path-to-linked-file
```

| Parameter | Explanation                                              | type   | default |
| ---       | ---                                                      | ---    | :---:   |
| path      | The file path to linked file                             | string | \\      |
| exist     | only create the link if the original file exists| bool   | true   |
