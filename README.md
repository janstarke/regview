# regview
Offline-viewer for registry files

<img src="https://github.com/janstarke/regview/blob/master/doc/regview_sample.png?raw=true">

# Installation

```shell
sudo apt install libncurses-dev
cargo install regview
```

# Usage

```shell
Offline-viewer for registry files

Usage: regview [OPTIONS] <HIVE_FILE>

Arguments:
  <HIVE_FILE>  path to registry hive file

Options:
  -L, --log <LOGFILES>     transaction LOG file(s). This argument can be specified one or two times
  -I, --ignore-base-block  ignore the base block (e.g. if it was encrypted by some ransomware)
  -h, --help               Print help
  -V, --version            Print version
```
