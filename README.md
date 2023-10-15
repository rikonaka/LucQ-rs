# LucQ-rs

Linux user command queue, help Linux users execute commands (only `Linux`) or codes (`Shell` and `Python`) in sequence.

## Usage

```bash
Linux user command queue

Usage: lucq [OPTIONS]

Options:
  -m, --mode <MODE>      Run mode (cli or exec) [default: cli]
  -a, --add <ADD>        Add one command [default: null]
  -r, --remove <REMOVE>  Remove one command [default: null]
  -l, --list             List all commands
  -c, --clean            Clean database
  -h, --help             Print help
  -V, --version          Print version
```

### Prepare two terminals with code environments

Operations such as activating the python environment in the window or `tmux`, etc.

### Execute in the first window or tmux

```bash
lucq --mode exec
```

This will activate the `executor`, waiting for the user to add a command.

### Execute in the second window or tmux

We add the code or commands we want to execute sequentially in the second window or `tmux`.

Short command

```bash
lucq --add ls
```

Long command

```bash
lucq --add "test.py -a 1"
```

Specify executor

```bash
lucq --add test.py --executor /home/riko/venv/bin/python
```

Show progress

```bash
lucq --list
```

Outputs

```bash
S | Jobs
o | id[1], user[riko], add_time[2023-10-14 15:41:21], used_time[00:00:10], command[test.py]
o | id[2], user[riko], add_time[2023-10-14 15:41:22], used_time[00:00:10], command[test.py]
o | id[3], user[riko], add_time[2023-10-14 15:41:22], used_time[00:00:10], command[test.py]
o | id[4], user[riko], add_time[2023-10-14 15:46:45], used_time[00:00:10], command[test.py -a 1]
o | id[5], user[riko], add_time[2023-10-14 15:47:29], used_time[00:00:10], command[test.py -a 1]
o | id[6], user[riko], add_time[2023-10-14 15:47:35], used_time[00:00:10], command[test.py -a 1]
r | id[7], user[riko], add_time[2023-10-14 15:47:36], used_time[00:00:00], command[test.py -a 1]
x | id[8], user[riko], add_time[2023-10-14 15:48:48], used_time[00:00:00], command[test.py -a 1]
x | id[9], user[riko], add_time[2023-10-14 15:49:01], used_time[00:00:00], command[test.py -a 1]
```

- `o` means command execute finished
- `r` means command still running
- `x` means command not started
- `e` means command quit with error
- `e` means command cancel by user


### Remove command from queue

Use `--list` to find out command id (example 9) then

```bash
lucq --remove 9
```

### Clean database

LucQ use sqlite to store the command (`$HOME/lucq.sql`)

```bash
lucq --clean
```