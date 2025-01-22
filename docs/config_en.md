# FuckRun Configuration Guide ⚙️

This document details the configuration file format and all available options in FuckRun.

## Table of Contents
- [Configuration File Format](#configuration-file-format)
- [Global Configuration](#global-configuration)
- [Process Configuration](#process-configuration)
- [Log Configuration](#log-configuration)
- [File System Configuration](#file-system-configuration)
- [State Management Configuration](#state-management-configuration)
- [Example Configuration](#example-configuration)

## Configuration File Format

FuckRun supports both YAML and JSON configuration file formats. YAML is used by default.

Configuration file search order:
1. `config.yaml` in the process directory
2. Process configuration in `.fuckrun` directory
3. `config.yaml` in the project root directory

## Global Configuration

Global configuration applies to all processes and can be overridden by specific process configurations.

```yaml
global:
  # Global working directory
  working_dir: /app
  
  # Global environment variables
  env:
    RUST_LOG: info
    
  # Global log configuration
  log:
    file: app.log
    level: info
    max_size: 100  # MB
    max_files: 5
    
  # File system configuration
  fs:
    max_retries: 3
    retry_delay_ms: 100
    default_mode: 644
    exit_wait_ms: 500
    default_file_mode: 644
    default_dir_mode: 755
    
  # State management configuration
  state:
    state_dir: .fuckrun
    state_filename: state.json
    default_working_dir: .
    
  # Process management configuration
  process:
    default_port: 5000
    init_wait_secs: 5
    health_check_timeout_secs: 5
    health_check_retries: 10
    retry_interval_secs: 2
    graceful_shutdown_timeout_secs: 3
    exit_wait_ms: 500
    default_python_interpreter: python
    default_script_path: examples/simple_web.py
```

## Process Configuration

Each process can have its own independent configuration.

```yaml
processes:
  web:  # Process name
    # Required fields
    program: python         # Executable path
    args: [app.py]         # Command line arguments
    
    # Optional fields (with defaults)
    working_dir: .         # Working directory
    auto_restart: false    # Auto restart on crash
    start_delay: 0         # Start delay (seconds)
    max_restarts: 3        # Maximum restart attempts
    
    # Optional fields (no defaults)
    depends_on: []         # Process dependencies
    health_check_url: ~    # Health check URL
    env: {}               # Environment variables
    
    # Log configuration
    log:
      file: ~             # Log file path
      level: info         # Log level
      max_size: 100       # Maximum file size (MB)
      max_files: 5        # Number of files to keep
```

## Log Configuration

Log configuration can be set at both global and process levels.

```yaml
log:
  # Log file path (optional)
  file: app.log
  
  # Log level (default: info)
  # Available values: trace, debug, info, warn, error
  level: info
  
  # Maximum single log file size (MB) (default: 100)
  max_size: 100
  
  # Number of log files to keep (default: 5)
  max_files: 5
```

## File System Configuration

File system related configuration options.

```yaml
fs:
  # Maximum retry attempts for file operations (default: 3)
  max_retries: 3
  
  # Retry delay in milliseconds (default: 100)
  retry_delay_ms: 100
  
  # Default file permissions (default: 644)
  default_mode: 644
  
  # Process exit wait time in milliseconds (default: 500)
  exit_wait_ms: 500
  
  # Default file permission mask (default: 644)
  default_file_mode: 644
  
  # Default directory permission mask (default: 755)
  default_dir_mode: 755
```

## State Management Configuration

Process state management related configuration.

```yaml
state:
  # State file directory (default: .fuckrun)
  state_dir: .fuckrun
  
  # State filename (default: state.json)
  state_filename: state.json
  
  # Default working directory (default: .)
  default_working_dir: .
```

## Process Management Configuration

Process management related configuration options.

```yaml
process:
  # Default port number (default: 5000)
  default_port: 5000
  
  # Process initialization wait time in seconds (default: 5)
  init_wait_secs: 5
  
  # Health check timeout in seconds (default: 5)
  health_check_timeout_secs: 5
  
  # Health check retry attempts (default: 10)
  health_check_retries: 10
  
  # Retry interval in seconds (default: 2)
  retry_interval_secs: 2
  
  # Graceful shutdown timeout in seconds (default: 3)
  graceful_shutdown_timeout_secs: 3
  
  # Process exit wait time in milliseconds (default: 500)
  exit_wait_ms: 500
  
  # Default Python interpreter (default: python)
  default_python_interpreter: python
  
  # Default Python script path
  default_script_path: examples/simple_web.py
```

## Example Configuration

Here's a complete configuration file example with multiple process configurations:

```yaml
# Global configuration
global:
  working_dir: /app
  env:
    RUST_LOG: info
  log:
    level: info
    max_size: 200
    max_files: 10

# Process configurations
processes:
  # Web service
  web:
    program: python
    args: [app.py]
    working_dir: ./web
    auto_restart: true
    health_check_url: http://localhost:8000/health
    env:
      PORT: 8000
      DEBUG: true
    log:
      level: debug
      
  # API service
  api:
    program: node
    args: [server.js]
    working_dir: ./api
    auto_restart: true
    start_delay: 5
    depends_on: [web]
    health_check_url: http://localhost:3000/health
    env:
      PORT: 3000
      NODE_ENV: production
      
  # Background worker
  worker:
    program: python
    args: [worker.py]
    working_dir: ./worker
    auto_restart: true
    env:
      QUEUE_URL: redis://localhost:6379
    log:
      level: info
      max_size: 500
      max_files: 7
``` 