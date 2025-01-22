# FuckRun Troubleshooting Guide 🔧

This document helps you solve common issues you may encounter while using FuckRun.

## Table of Contents
- [Common Errors](#common-errors)
- [Log Viewing](#log-viewing)
- [Process Management](#process-management)
- [Configuration Issues](#configuration-issues)
- [System Related](#system-related)

## Common Errors

### 1. Process Start Failure

#### Symptoms
- Start command returns error
- Process status shows not running
- Error messages in logs

#### Possible Causes
1. Incorrect program path
2. Working directory doesn't exist
3. Permission issues
4. Port in use
5. Python interpreter issues

#### Solutions

1. Check program path
```bash
# Verify program exists
ls <program_path>

# Check Python interpreter
python --version
```

2. Check working directory
```bash
# Verify directory exists
ls <working_dir>

# Create directory
mkdir -p <working_dir>
```

3. Check permissions
```bash
# View file permissions
ls -l <program_path>

# Modify permissions
chmod +x <program_path>
```

4. Check ports
```bash
# Windows
netstat -ano | findstr <port>

# Linux
netstat -tulpn | grep <port>
```

5. Python related
```bash
# Check dependencies
pip list

# Install dependencies
pip install -r requirements.txt
```

### 2. Health Check Failure

#### Symptoms
- Process starts but health check fails
- Logs show connection timeout or refused

#### Possible Causes
1. Program not fully started
2. Incorrect health check URL
3. Firewall blocking
4. Internal program error

#### Solutions

1. Adjust wait times
```yaml
process:
  init_wait_secs: 10
  health_check_timeout_secs: 10
```

2. Verify health check
```bash
# Manual health check test
curl http://localhost:<port>/health
```

3. Check firewall
```bash
# Windows
netsh advfirewall show currentprofile

# Linux
sudo ufw status
```

4. Health check always fails
   - Verify URL is correct
   - Confirm port is correct
   - Check application logs for errors
   - Check firewall settings
   - **Check if proxy (like Clash) is causing failures (especially 502 Bad Gateway)**

### 3. Auto Restart Issues

#### Symptoms
- Process restarts frequently
- Restart count exceeds limit
- Cannot exit normally

#### Possible Causes
1. Memory leak
2. Misconfiguration
3. Insufficient system resources
4. Dependencies not ready

#### Solutions

1. Adjust restart strategy
```yaml
processes:
  web:
    auto_restart: true
    max_restarts: 5
    start_delay: 5
```

2. Monitor resource usage
```bash
# View process status
fuckrun list --watch

# System resources
top  # Linux
taskmgr  # Windows
```

## Log Viewing

### System Logs

```bash
# View FuckRun system logs
fuckrun logs --system

# Real-time tracking
fuckrun logs --system -f
```

### Process Logs

```bash
# View standard output
fuckrun logs -n <process_name>

# View error logs
fuckrun logs -n <process_name> --type stderr

# View logs for specific date
fuckrun logs -n <process_name> --date 2024-01-22
```

### Log Locations

- System logs: `.fuckrun/logs/<date>/fuckrun.log`
- Process logs: `.fuckrun/processes/<process_name>/logs/<date>/{stdout,stderr}.log`

## Process Management

### 1. Process Status Abnormal

#### Symptoms
- Status shows running but actually not
- PID mismatch
- Cannot stop process

#### Solutions

1. Reset process state
```bash
# Stop process
fuckrun stop -n <process_name>

# Clean state file
rm -f .fuckrun/processes/<process_name>/state.json

# Restart
fuckrun start -n <process_name>
```

2. Force stop
```bash
# Windows
taskkill /F /PID <pid>

# Linux
kill -9 <pid>
```

### 2. Daemon Process Issues

#### Symptoms
- Child process remains after main process exits
- Cannot stop daemon normally
- Abnormal log output

#### Solutions

1. Check process tree
```bash
# Windows
wmic process where ParentProcessId=<pid> get ProcessId,CommandLine

# Linux
pstree -p <pid>
```

2. Clean zombie processes
```bash
# Find related processes
ps aux | grep <process_name>

# Stop all related processes
pkill -f <process_name>
```

## Configuration Issues

### 1. Configuration File Load Failure

#### Symptoms
- Configuration file not found
- Configuration not taking effect
- Format errors

#### Solutions

1. Check configuration file location
```bash
# View configuration file
cat config.yaml

# Validate YAML syntax
python -c "import yaml; yaml.safe_load(open('config.yaml'))"
```

2. Use default configuration
```bash
# Copy example configuration
cp config.yaml.example config.yaml

# Edit configuration
vim config.yaml
```

### 2. Environment Variable Issues

#### Symptoms
- Program cannot access environment variables
- Environment variables not taking effect
- Incorrect values

#### Solutions

1. Check environment variable configuration
```yaml
processes:
  web:
    env:
      PORT: 8000
      DEBUG: true
```

2. Verify environment variables
```bash
# Print environment variables at startup
fuckrun start -n web --env DEBUG=true
```

## System Related

### 1. Permission Issues

#### Windows Specific Issues
- Requires administrator privileges
- UAC restrictions
- Firewall blocking

#### Solutions
1. Run as administrator
2. Adjust UAC settings
3. Add firewall rules

#### Linux Specific Issues
- File permissions
- SELinux restrictions
- systemd integration

#### Solutions
1. Adjust file permissions
```bash
chmod +x /path/to/program
chown user:group /path/to/program
```

2. SELinux settings
```bash
# Check SELinux status
sestatus

# Allow network access
semanage port -a -t http_port_t -p tcp <port>
```

### 2. Resource Limits

#### Symptoms
- Insufficient memory
- High CPU usage
- File descriptor exhaustion

#### Solutions

1. Check system limits
```bash
# View limits
ulimit -a

# Adjust limits
ulimit -n 65535  # file descriptors
```

2. Monitor resource usage
```bash
# Real-time monitoring
fuckrun list --watch --filter min-cpu=50

# View detailed status
fuckrun status -n <process_name>
```

3. Optimize configuration
```yaml
process:
  # Adjust process priority
  priority: normal
  
  # Limit resource usage
  max_memory: 1024m
  max_cpu: 50
``` 