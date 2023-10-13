# LucQ-rs

Linux user command queue, help Linux users execute commands or codes in sequence.

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

Short command

```bash
lucq --add ls
```

Long command

```bash
lucq --add "test.py -a 1"
```

Show progress

```bash
lucq --list
```

Outputs

```bash
S | Jobs
o | id[1], user[riko], add_time[2023-10-14 00:41:21 +08:00], used_time[0:0:10], command[test.py]
o | id[2], user[riko], add_time[2023-10-14 00:41:22 +08:00], used_time[0:0:10], command[test.py]
o | id[3], user[riko], add_time[2023-10-14 00:41:22 +08:00], used_time[0:0:10], command[test.py]
o | id[4], user[riko], add_time[2023-10-14 00:46:45 +08:00], used_time[0:0:10], command[test.py -a 1]
o | id[5], user[riko], add_time[2023-10-14 00:47:29 +08:00], used_time[0:0:10], command[test.py -a 1]
o | id[6], user[riko], add_time[2023-10-14 00:47:35 +08:00], used_time[0:0:10], command[test.py -a 1]
o | id[7], user[riko], add_time[2023-10-14 00:47:36 +08:00], used_time[0:0:10], command[test.py -a 1]
o | id[8], user[riko], add_time[2023-10-14 00:48:48 +08:00], used_time[0:0:10], command[test.py -a 1]
o | id[9], user[riko], add_time[2023-10-14 00:49:01 +08:00], used_time[0:0:10], command[test.py -a 1]
```

- `o` means command exec finish
- `r` means command still running
- `x` means command not started
