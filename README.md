# LucQ-rs

Linux user command queue, help Linux users execute commands (only `Linux`) or codes (`Shell` and `Python`) in sequence.

## Usage

```bash
Linux user command queue

Usage: lucq [OPTIONS]

Options:
  -m, --mode <mode>      Run mode (cli or exec) [default: cli]
  -a, --add <job>        Add one command [default: null]
      --before <id>      Add one command before <id> [default: -1]
      --after <id>       Add one command after <id> [default: -1]
  -r, --remove <id(s)>   Remove command(s) (example: 1 or 1-5) [default: null]
  -c, --cancel <id(s)>   Cancel command(s) (keep it in history, example: 1 or 1-5) [default: null]
  -e, --executor <path>  Executor path (example: /usr/bin/python3) [default: null]
  -l, --list             List all commands
      --clean            Clean database
  -h, --help             Print help
  -V, --version          Print version
```

### Prepare two terminals

One for execute the command in background, one for add new command into queue.

### Execute in the first window

```bash
lucq --mode exec
```

This will activate the `executor`, waiting for the user to add a command.

### Execute in the second window

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
o | id[1], user[riko], add[10-14 15:41], start[10-14 15:41], used[00:00:10], command[test.py]
o | id[2], user[riko], add[10-14 15:41], start[10-14 15:51], used[00:00:10], command[test.py]
o | id[3], user[riko], add[10-14 15:41], start[10-14 16:01], used[00:00:10], command[test.py]
o | id[4], user[riko], add[10-14 15:46], start[10-14 15:11], used[00:00:10], command[test.py -a 1]
o | id[5], user[riko], add[10-14 15:47], start[10-14 15:21], used[00:00:10], command[test.py -a 1]
o | id[6], user[riko], add[10-14 15:47], start[10-14 15:31], used[00:00:10], command[test.py -a 1]
r | id[7], user[riko], add[10-14 15:47], start[10-14 15:41], used[00:00:00], command[test.py -a 1]
x | id[8], user[riko], add[10-14 15:48], start[00-00 00:00], used[00:00:00], command[test.py -a 1]
x | id[9], user[riko], add[10-14 15:49], start[00-00 00:00], used[00:00:00], command[test.py -a 1]
```

- `o` means command execute finished
- `r` means command still running
- `x` means command not started
- `e` means command quit with error
- `c` means command cancel by user


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