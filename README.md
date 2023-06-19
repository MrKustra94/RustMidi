Tool that allows to display status of common background tasks on MIDI controller pads

## Table of contents
- [Install](#install)
- [Usage](#usage)
- [How it works](#how-it-works)

## Install
Currently, there are no official binaries released. This tool must be compiled using Rust stack.
It can be installed using [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) utility:
```shell
cargo install --git https://github.com/MrKustra94/RustMidi
```
This should install a `rust_midi` executable in `~/.cargo/bin` directory.
Installation could be also done directly on sources via:
```shell
cargo build --release
```
Note that building process may take a couple of minutes.

## Usage
This tool requires a special configuration file in YAML format.
By default, `rust_midi` expects an existence of file named `midi_config.yaml`.
For an example of valid configuration file, please check [midi_config.yaml](midi_config.yaml) defined in this repository.
```shell
rust_midi -p midi_config.yaml
```

## How it works
`rust_midi` simply interprets passed YAML configuration file and schedules each defined action for an execution.
Currently, each task is running in an endless loop, until program is aborted.
The lifecycle of a single pad is managed by generic [actor](src/worker/actor.rs), which additionally may suspend/resume an action once pad is pressed.
There are two types of actions that can be mapped to the pad:
- [Kubernetes](src/worker/k8s.rs)
- [Script](src/worker/script.rs)
### Kubernetes
Kubernetes handler is making a call to Kubernetes API to check the status of the deployment.
Its handler is stateful - the output of next invocation is compared with an output of previous invocations.
### Script
Script handler is continuously making a call to system to execute passed command.
This handler is stateless - pad corresponding to Script handler is filled with a color matching to output of latest invocation.
