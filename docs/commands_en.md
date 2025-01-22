# FuckRun Command Guide 🎮

This document details all commands available in FuckRun and their usage.

## Table of Contents
- [start - Start Process](#start---start-process)
- [stop - Stop Process](#stop---stop-process)
- [status - Check Status](#status---check-status)
- [monitor - Monitor Process](#monitor---monitor-process)
- [logs - View Logs](#logs---view-logs)
- [list - List Processes](#list---list-processes)

## start - Start Process

Start a new process.

### Basic Usage

```bash
fuckrun start -n <process_name>
```

### Parameters

- `-n, --name <name>`: Process name
- `--python <interpreter_path>`: Specify Python interpreter path
- `--port <port>`: Specify port number (for default process)
- `--detach`: Detach after start (non-blocking terminal)
- `--daemon`: Run in daemon mode
- `--auto-restart`: Auto restart on crash

### Examples

```bash
# Start a process named web
fuckrun start -n web

# Start with auto restart
fuckrun start -n web --auto-restart

# Specify Python interpreter
fuckrun start -n web --python python3.9

# Run in daemon mode
fuckrun start -n web --daemon
```

## stop - Stop Process

Stop a running process.

### Basic Usage

```bash
fuckrun stop -n <process_name>
```

### Parameters

- `-n, --name <name>`: Process name

### Examples

```bash
# Stop process named web
fuckrun stop -n web

# Stop default process
fuckrun stop
```

## status - Check Status

Check the running status of a process.

### Basic Usage

```bash
fuckrun status -n <process_name>
```

### Parameters

- `-n, --name <name>`: Process name
- `--port <port>`: Specify port number (for default process)

### Examples

```bash
# Check web process status
fuckrun status -n web

# Check default process status
fuckrun status
```

## monitor - Monitor Process

Monitor process status and handle output.

### Basic Usage

```bash
fuckrun monitor -n <process_name> [options]
```

### Parameters

- `-n, --name <name>`: Process name
- `--program <program>`: Program to run
- `--args <args>`: Program arguments
- `--working-dir <dir>`: Working directory
- `--env <env_vars>`: Environment variables (format: KEY=VALUE)
- `--auto-restart`: Auto restart on crash

### Examples

```bash
# Monitor web process
fuckrun monitor -n web --program python --args app.py

# Monitor with environment variables
fuckrun monitor -n web --program python --args app.py --env PORT=8000
```

## logs - View Logs

View process output logs.

### Basic Usage

```bash
fuckrun logs -n <process_name> [options]
```

### Parameters

- `-n, --name <name>`: Process name
- `-f, --follow`: Follow log output in real-time
- `--type <type>`: Log type (stdout/stderr)
- `--date <date>`: Specify date (YYYY-MM-DD)

### Examples

```bash
# View web process stdout
fuckrun logs -n web

# Follow error logs in real-time
fuckrun logs -n web -f --type stderr

# View logs for specific date
fuckrun logs -n web --date 2024-01-22
```

### System Logs

View FuckRun's own logs:

```bash
# View system logs
fuckrun logs --system

# Follow system logs in real-time
fuckrun logs --system -f
```

## list - List Processes

List all running processes.

### Basic Usage

```bash
fuckrun list [options]
```

### Parameters

- `--app-dir <dir>`: Specify application directory
- `--json`: Output in JSON format
- `--watch`: Real-time monitoring mode
- `--filter <conditions>`: Filter conditions

### Filter Options

- `name=<name>`: Filter by name
- `status=<status>`: Filter by status
- `min-uptime=<seconds>`: Minimum uptime
- `max-uptime=<seconds>`: Maximum uptime
- `min-cpu=<percentage>`: Minimum CPU usage
- `max-cpu=<percentage>`: Maximum CPU usage
- `min-mem=<bytes>`: Minimum memory usage
- `max-mem=<bytes>`: Maximum memory usage

### Examples

```bash
# List all processes
fuckrun list

# Output in JSON format
fuckrun list --json

# Real-time monitoring
fuckrun list --watch

# Filter running processes
fuckrun list --filter status=running

# Filter high CPU usage processes
fuckrun list --filter min-cpu=50
```

### Output Fields

List mode displays the following information:

- `NAME`: Process name
- `PID`: Process ID
- `MONITOR`: Monitor process ID
- `STATUS`: Running status
- `RESTARTS`: Restart count
- `UPTIME`: Running time
- `CPU`: CPU usage
- `MEM`: Memory usage
- `LAST START`: Last start time 