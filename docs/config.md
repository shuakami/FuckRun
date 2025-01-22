# FuckRun 配置指南 ⚙️

本文档详细介绍了 FuckRun 的配置文件格式和所有可用选项。

## 目录
- [配置文件格式](#配置文件格式)
- [全局配置](#全局配置)
- [进程配置](#进程配置)
- [日志配置](#日志配置)
- [文件系统配置](#文件系统配置)
- [状态管理配置](#状态管理配置)
- [示例配置](#示例配置)

## 配置文件格式

FuckRun 支持 YAML 和 JSON 两种配置文件格式。默认使用 YAML 格式。

配置文件搜索顺序：
1. 进程目录下的 `config.yaml`
2. `.fuckrun` 目录下的进程配置
3. 项目根目录的 `config.yaml`

## 全局配置

全局配置会应用到所有进程，可以被具体进程的配置覆盖。

```yaml
global:
  # 全局工作目录
  working_dir: /app
  
  # 全局环境变量
  env:
    RUST_LOG: info
    
  # 全局日志配置
  log:
    file: app.log
    level: info
    max_size: 100  # MB
    max_files: 5
    
  # 文件系统配置
  fs:
    max_retries: 3
    retry_delay_ms: 100
    default_mode: 644
    exit_wait_ms: 500
    default_file_mode: 644
    default_dir_mode: 755
    
  # 状态管理配置
  state:
    state_dir: .fuckrun
    state_filename: state.json
    default_working_dir: .
    
  # 进程管理配置
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

## 进程配置

每个进程可以有自己的独立配置。

```yaml
processes:
  web:  # 进程名称
    # 必填项
    program: python         # 可执行文件路径
    args: [app.py]         # 命令行参数
    
    # 可选项(带默认值)
    working_dir: .         # 工作目录
    auto_restart: false    # 是否自动重启
    start_delay: 0         # 启动延迟(秒)
    max_restarts: 3        # 最大重启次数
    
    # 可选项(无默认值)
    depends_on: []         # 依赖的其他进程
    health_check_url: ~    # 健康检查URL
    env: {}               # 环境变量
    
    # 日志配置
    log:
      file: ~             # 日志文件路径
      level: info         # 日志级别
      max_size: 100       # 最大文件大小(MB)
      max_files: 5        # 保留文件数量
```

## 日志配置

日志配置可以在全局和进程级别设置。

```yaml
log:
  # 日志文件路径(可选)
  file: app.log
  
  # 日志级别(默认:info)
  # 可选值: trace, debug, info, warn, error
  level: info
  
  # 单个日志文件最大大小(MB)(默认:100)
  max_size: 100
  
  # 保留的日志文件数量(默认:5)
  max_files: 5
```

## 文件系统配置

文件系统相关的配置选项。

```yaml
fs:
  # 文件操作最大重试次数(默认:3)
  max_retries: 3
  
  # 重试延迟时间(毫秒)(默认:100)
  retry_delay_ms: 100
  
  # 默认文件权限(默认:644)
  default_mode: 644
  
  # 进程退出等待时间(毫秒)(默认:500)
  exit_wait_ms: 500
  
  # 默认文件权限掩码(默认:644)
  default_file_mode: 644
  
  # 默认目录权限掩码(默认:755)
  default_dir_mode: 755
```

## 状态管理配置

进程状态管理相关的配置。

```yaml
state:
  # 状态文件目录(默认:.fuckrun)
  state_dir: .fuckrun
  
  # 状态文件名(默认:state.json)
  state_filename: state.json
  
  # 默认工作目录(默认:.)
  default_working_dir: .
```

## 进程管理配置

进程管理相关的配置选项。

```yaml
process:
  # 默认端口号(默认:5000)
  default_port: 5000
  
  # 进程初始化等待时间(秒)(默认:5)
  init_wait_secs: 5
  
  # 健康检查超时时间(秒)(默认:5)
  health_check_timeout_secs: 5
  
  # 健康检查重试次数(默认:10)
  health_check_retries: 10
  
  # 重试间隔时间(秒)(默认:2)
  retry_interval_secs: 2
  
  # 优雅关闭超时时间(秒)(默认:3)
  graceful_shutdown_timeout_secs: 3
  
  # 进程退出等待时间(毫秒)(默认:500)
  exit_wait_ms: 500
  
  # 默认Python解释器(默认:python)
  default_python_interpreter: python
  
  # 默认Python脚本路径
  default_script_path: examples/simple_web.py
```

## 示例配置

这是一个完整的配置文件示例，包含了多个进程的配置：

```yaml
# 全局配置
global:
  working_dir: /app
  env:
    RUST_LOG: info
  log:
    level: info
    max_size: 200
    max_files: 10

# 进程配置
processes:
  # Web服务
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
      
  # API服务
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
      
  # 后台任务
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