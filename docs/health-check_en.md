# FuckRun Health Check Guide 🏥

This document details the health check mechanism in FuckRun.

## Table of Contents
- [Health Check Overview](#health-check-overview)
- [Configuration Options](#configuration-options)
- [How It Works](#how-it-works)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Health Check Overview

Health check is a crucial mechanism in FuckRun for monitoring process status. It confirms process health by periodically accessing an HTTP endpoint provided by the process.

Key features:
- Ensure process starts properly
- Monitor process running status
- Support automatic restart
- Provide status reporting

## Configuration Options

Health check related configuration options:

```yaml
# Process configuration
processes:
  web:
    # Health check URL (required)
    health_check_url: http://localhost:8000/health
    
    # Process management configuration (optional)
    process:
      # Health check timeout in seconds (default: 5)
      health_check_timeout_secs: 5
      
      # Health check retry attempts (default: 10)
      health_check_retries: 10
      
      # Retry interval in seconds (default: 2)
      retry_interval_secs: 2
```

### Configuration Details

1. `health_check_url`
   - HTTP endpoint for health checks
   - Must return 2xx status code
   - Recommended to use a dedicated health check endpoint

2. `health_check_timeout_secs`
   - Timeout for a single health check
   - Check fails if timeout exceeded
   - Default is 5 seconds

3. `health_check_retries`
   - Number of retry attempts after failed check
   - Process considered unhealthy only after all retries fail
   - Default is 10 attempts

4. `retry_interval_secs`
   - Wait time between retries
   - Prevents overwhelming service with frequent checks
   - Default is 2 seconds

## How It Works

### Startup Check

1. After process starts, FuckRun waits for `init_wait_secs`
2. Begin health checks:
   - Send HTTP GET request to `health_check_url`
   - Wait up to `health_check_timeout_secs`
   - Retry if timeout or failure occurs
3. Retry mechanism:
   - Maximum `health_check_retries` attempts
   - Wait `retry_interval_secs` between attempts
   - Startup fails if all retries fail

### Runtime Check

1. Regular status checks:
   - Verify process exists
   - Access health check endpoint
2. Error handling:
   - Failed checks are logged
   - Process restarts if auto-restart enabled
3. Status updates:
   - Check results update state file
   - View via `status` command

## Best Practices

### Health Check Endpoint Design

1. Lightweight check
```python
@app.route('/health')
def health_check():
    return {'status': 'ok'}, 200
```

2. Deep check
```python
@app.route('/health')
def health_check():
    try:
        # Check database connection
        db.ping()
        
        # Check cache service
        cache.ping()
        
        # Check message queue
        queue.ping()
        
        return {
            'status': 'ok',
            'database': 'connected',
            'cache': 'connected',
            'queue': 'connected'
        }, 200
    except Exception as e:
        return {
            'status': 'error',
            'error': str(e)
        }, 500
```

### Configuration Recommendations

1. Timeout settings
   - `health_check_timeout_secs`: Set to 2-3 times normal response time
   - `init_wait_secs`: Adjust based on application startup time

2. Retry strategy
   - `health_check_retries`: Recommend 10-20 attempts
   - `retry_interval_secs`: Recommend 2-5 seconds

3. Auto restart
   - Enable `auto_restart` recommended
   - Set reasonable `max_restarts` to prevent infinite restarts

## Troubleshooting

### Common Issues

1. Health check always fails
   - Verify URL is correct
   - Confirm port is correct
   - Check application logs for errors
   - Check firewall settings
   - **Check if proxy (like Clash) is causing failures (especially 502 Bad Gateway)**

2. Check timeout
   - Increase `health_check_timeout_secs`
   - Optimize health check endpoint performance
   - Check system resource usage

3. Frequent restarts
   - Check application logs for crash cause
   - Adjust `retry_interval_secs` to increase interval
   - Consider increasing `max_restarts`

### Debugging Methods

1. Enable detailed logging
```yaml
global:
  log:
    level: debug
```

2. Manual health check test
```bash
# Test with curl
curl http://localhost:8000/health

# Check process status
fuckrun status -n web
```

3. View logs
```bash
# View process logs
fuckrun logs -n web

# View system logs
fuckrun logs --system
``` 