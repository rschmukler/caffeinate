# Caffeinate

A command-line app bringing caffeinate functionality to [xidlehook](https://github.com/jD91mZM2/xidlehook).

## Features

### Triggers
Triggers are used to monitor for when caffeinate should exit (and allow
`xidlehook` to resume).

- [x] Timer
- [x] PID-based monitoring

### Quit Actions
Quit actions are used in conjunction with triggers to perform a final action
before caffeinate exits.


## Setup


```sh
cargo install --git https://github.com/rschmukler/caffeinate
```

Start xidlehook with a socket argument:

```sh
xidlehook --timer primary 60 "xset dpms force off" --socket "/tmp/xidlehook.sock"
```


## Usage Examples

```sh
# Running indefinitely (exit with Ctrl-C)
caffeinate

# Running for an hour
caffeinate --timer 3600

# Running until a process exits, then shutdown down the machine
caffeinate --pid 1234 --quit=shutdown
```
