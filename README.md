# FuckRun 🚀

[![简体中文](https://img.shields.io/badge/简体中文-red.svg?style=flat-square)](./README_ZH_CN.md)
[![Version](https://img.shields.io/badge/version-1.0.0-FBC02D.svg?style=flat-square)]()
[![Build](https://img.shields.io/badge/build-passing-FF9800.svg?style=flat-square)]()
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-2196F3.svg?style=flat-square)]()

> Welcome to the world of efficient program management! FuckRun is here to be your trusty assistant—taking care of running your processes, managing logs, and ensuring smooth operation with minimal effort.

## Why Do You Need FuckRun? 

As your project grows and you run more processes, you might find yourself overwhelmed by:
- Multiple terminal windows for different tasks
- Difficulty locating logs for each program
- Sudden crashes without notifications
- The hassle of manually restarting programs

That's where **FuckRun** steps in—streamlining everything into a well-organized system:
- Manage all your processes from one place
- Auto-record logs for every action
- Instant notifications when something goes wrong
- Automatically restart programs if needed

## Unified Output Format 

To make integration easier, **FuckRun** uses a simple, consistent output format for program statuses, allowing external programs (like Python) to easily parse and handle the data.

### Output Format Explanation

Every status message follows this format:
```
STATUS:ACTION:NAME[:PID]
```

Where:
- `STATUS` is a fixed prefix, indicating the message is a status update.
- `ACTION` describes the action taken, such as:
  - `STARTING`: Program is starting
  - `STARTED`: Program has started (includes PID)
  - `STOPPING`: Program is stopping
  - `STOPPED`: Program has stopped
  - `RUNNING`: Program is running (includes PID)
- `NAME` is the program's name.
- `PID` is the process ID (only for `STARTED` and `RUNNING` statuses).

### Python Parsing Example

```python
def parse_fuckrun_status(line):
    """Parses the output from FuckRun"""
    if not line.startswith("STATUS:"):
        return None
        
    parts = line.strip().split(":")
    if len(parts) < 3:
        return None
        
    status = {
        "action": parts[1],
        "name": parts[2],
        "pid": int(parts[3]) if len(parts) > 3 else None
    }
    return status

# Example Usage
import subprocess

# Start a program
process = subprocess.Popen(
    ["fuckrun", "start", "-n", "web"], 
    stdout=subprocess.PIPE,
    text=True
)

# Read the output
for line in process.stdout:
    status = parse_fuckrun_status(line)
    if status:
        print(f"Program {status['name']} {status['action']}")
        if status['pid']:
            print(f"PID: {status['pid']}")
```

### Example Status Output

```bash
# When starting a program
STATUS:STARTING:web
STATUS:STARTED:web:1234

# When checking the status
STATUS:RUNNING:web:1234

# When stopping a program
STATUS:STOPPING:web
STATUS:STOPPED:web

# If an error occurs
Error: [Error message] 
```

## Getting Started - 5-Minute Quickstart 🏃

> ⚠️ Before you begin:
> 1. Ensure you have [Git](https://git-scm.com/) installed
> 2. Avoid paths with spaces or Chinese characters
> 3. If you encounter permission issues, try running as Administrator

### Step 1: Install FuckRun

Open your terminal (press Win+R and type `cmd` on Windows), and run:

```bash
# 1. Clone the repository
git clone https://github.com/yourusername/fuckrun

# 2. Enter the directory
cd fuckrun

# 3. Build and install
cargo build --release
```

> 💡 If the build is slow, you can [download the precompiled version](https://github.com/yourusername/fuckrun/releases)

### Step 2: Create a Configuration File

Create a `config.yaml` in your project directory:

```yaml
# config.yaml
processes:
  # 'web' is the name of the program, you can change it
  web:
    # Use python to run app.py
    program: python
    args: ["app.py"]
    # Working directory for the program
    working_dir: ./app
    # Automatically restart on crash
    auto_restart: true
```

> 💡 More advanced configuration options are explained later.

### Step 3: Start Your Program

```bash
# Start the program named 'web'
fuckrun start -n web
```

You're done! Now, FuckRun will manage your program.

Want to check the status of your program? Try these commands:
```bash
# Check the program status
fuckrun status -n web

# View program logs
fuckrun logs -n web -f
```

## What's Next?

- 👉 [Learn about all the commands](docs/commands_en.md)
- 👉 [Complete guide to the configuration file](docs/config_en.md)
- 👉 [Health check configuration tutorial](docs/health-check_en.md)
- 👉 [Common troubleshooting solutions](docs/troubleshooting_en.md)

> For Chinese documentation:
> - 👉 [命令使用指南](docs/commands.md)
> - 👉 [配置文件完整指南](docs/config.md)
> - 👉 [健康检查配置教程](docs/health-check.md)
> - 👉 [常见问题解决方案](docs/troubleshooting.md)

## Understanding How FuckRun Works 🔍

> Knowing how FuckRun operates will help you use it more effectively.

The core concept behind **FuckRun** is simple: it's like a caretaker for your programs. Here's how it works:

1. **Starting Programs**: When you instruct **FuckRun** to start a program:
   - It creates an independent process to run your program
   - Collects the program's output into log files
   - Records the program's status (PID, start time, etc.)

2. **Monitoring**: While the program runs, **FuckRun** will:
   - Regularly check if the program is still running
   - Log the program's output
   - Alert you immediately if something goes wrong

3. **Auto Recovery**: If the program encounters an issue, **FuckRun** will:
   - Log the error
   - Attempt to restart the program
   - Notify you of what happened

Here's an overview of the process:
```ascii
[Your Program] <─────┐
                    │
                    ▼
              [FuckRun Assistant]
                    │
          ┌────────┴────────┐
          ▼                ▼
    [Manage Program]    [Log Outputs]
    - Start/Stop        - Standard Logs
    - Health Checks     - Error Logs
    - Auto-restart      - System Logs
```

## Common Commands Guide ⌨️

> This section teaches you the most commonly used **FuckRun** commands with examples.

### Start a Program

```bash
# Basic start command
fuckrun start -n web

# Start and view logs
fuckrun start -n web --tail

# Start multiple programs
fuckrun start -n web -n api -n worker

# Specify application directory
fuckrun start -n web --app-dir ./deployments/web/app

# Auto restart and specify directory
fuckrun start -n web --app-dir ./deployments/web/app --daemon --auto-restart
```

### Stop a Program

```bash
# Stop normally
fuckrun stop -n web

# Force stop (use only if the program is unresponsive)
fuckrun stop -n web --force

# Stop all programs
fuckrun stop --all
```

### Check Status

```bash
# Check a single program's status
fuckrun status -n web

# Check all programs' status
fuckrun status --all

# Output status in JSON format (useful for programmatic handling)
fuckrun status -n web --json
```

### View Logs

```bash
# View the latest logs
fuckrun logs -n web

# View logs in real-time
fuckrun logs -n web -f

# View only error logs
fuckrun logs -n web --error

# View the last 100 lines of logs
fuckrun logs -n web -n 100
```

> 💡 Add `-f` to follow logs in real-time, similar to `tail -f`.

Want to know more commands? You can:
- Run `fuckrun help` to see all commands
- Run `fuckrun help <command>` to get help on specific commands
- Check out the [full command documentation](docs/commands_en.md)
