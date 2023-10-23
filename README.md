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

Add command before id 3

```bash
lucq --add "test.py -a 1" --before 3
```

Add command after id 3

```bash
lucq --add "test.py -a 1" --after 3
```

Show progress

```bash
lucq --list
```

- 😐 or `x` means command is watting
- 😁 or `o` means command was finished
- 🥵 or `r` means command is running
- 😨 or `e` means command was error
- 🤡 or `c` means command was canceled

Disable emoji show

```bash
lucq --list --noemoji
```

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