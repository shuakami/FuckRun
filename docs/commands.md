# FuckRun 命令指南 🎮

本文档详细介绍了 FuckRun 的所有命令及其用法。

## 目录
- [start - 启动进程](#start---启动进程)
- [stop - 停止进程](#stop---停止进程)
- [status - 查看状态](#status---查看状态)
- [monitor - 监控进程](#monitor---监控进程)
- [logs - 查看日志](#logs---查看日志)
- [list - 列出进程](#list---列出进程)

## start - 启动进程

启动一个新的进程。

### 基本用法

```bash
fuckrun start -n <进程名称>
```

### 参数说明

- `-n, --name <名称>`: 进程名称
- `--python <解释器路径>`: 指定Python解释器路径
- `--port <端口>`: 指定端口号(默认进程使用)
- `--detach`: 启动后分离(不阻塞终端)
- `--daemon`: 以守护进程模式运行
- `--auto-restart`: 进程崩溃时自动重启

### 示例

```bash
# 启动名为web的进程
fuckrun start -n web

# 启动并自动重启
fuckrun start -n web --auto-restart

# 指定Python解释器
fuckrun start -n web --python python3.9

# 守护进程模式
fuckrun start -n web --daemon
```

## stop - 停止进程

停止一个正在运行的进程。

### 基本用法

```bash
fuckrun stop -n <进程名称>
```

### 参数说明

- `-n, --name <名称>`: 进程名称

### 示例

```bash
# 停止名为web的进程
fuckrun stop -n web

# 停止默认进程
fuckrun stop
```

## status - 查看状态

查看进程的运行状态。

### 基本用法

```bash
fuckrun status -n <进程名称>
```

### 参数说明

- `-n, --name <名称>`: 进程名称
- `--port <端口>`: 指定端口号(用于默认进程)

### 示例

```bash
# 查看web进程状态
fuckrun status -n web

# 查看默认进程状态
fuckrun status
```

## monitor - 监控进程

监控进程运行状态并处理输出。

### 基本用法

```bash
fuckrun monitor -n <进程名称> [选项]
```

### 参数说明

- `-n, --name <名称>`: 进程名称
- `--program <程序>`: 要运行的程序
- `--args <参数>`: 程序参数
- `--working-dir <目录>`: 工作目录
- `--env <环境变量>`: 环境变量(格式: KEY=VALUE)
- `--auto-restart`: 进程崩溃时自动重启

### 示例

```bash
# 监控web进程
fuckrun monitor -n web --program python --args app.py

# 带环境变量的监控
fuckrun monitor -n web --program python --args app.py --env PORT=8000
```

## logs - 查看日志

查看进程的输出日志。

### 基本用法

```bash
fuckrun logs -n <进程名称> [选项]
```

### 参数说明

- `-n, --name <名称>`: 进程名称
- `-f, --follow`: 实时跟踪日志
- `--type <类型>`: 日志类型(stdout/stderr)
- `--date <日期>`: 指定日期(YYYY-MM-DD)

### 示例

```bash
# 查看web进程的标准输出
fuckrun logs -n web

# 实时跟踪错误日志
fuckrun logs -n web -f --type stderr

# 查看指定日期的日志
fuckrun logs -n web --date 2024-01-22
```

### 系统日志

查看FuckRun自身的日志：

```bash
# 查看系统日志
fuckrun logs --system

# 实时跟踪系统日志
fuckrun logs --system -f
```

## list - 列出进程

列出所有运行中的进程。

### 基本用法

```bash
fuckrun list [选项]
```

### 参数说明

- `--app-dir <目录>`: 指定应用程序目录
- `--json`: 以JSON格式输出
- `--watch`: 实时监控模式
- `--filter <条件>`: 筛选条件

### 筛选选项

- `name=<名称>`: 按名称筛选
- `status=<状态>`: 按状态筛选
- `min-uptime=<秒>`: 最小运行时间
- `max-uptime=<秒>`: 最大运行时间
- `min-cpu=<百分比>`: 最小CPU使用率
- `max-cpu=<百分比>`: 最大CPU使用率
- `min-mem=<字节>`: 最小内存使用
- `max-mem=<字节>`: 最大内存使用

### 示例

```bash
# 列出所有进程
fuckrun list

# JSON格式输出
fuckrun list --json

# 实时监控
fuckrun list --watch

# 筛选运行中的进程
fuckrun list --filter status=running

# 筛选高CPU使用率进程
fuckrun list --filter min-cpu=50
```

### 输出说明

列表模式下会显示以下信息：

- `NAME`: 进程名称
- `PID`: 进程ID
- `MONITOR`: 监控进程ID
- `STATUS`: 运行状态
- `RESTARTS`: 重启次数
- `UPTIME`: 运行时间
- `CPU`: CPU使用率
- `MEM`: 内存使用
- `LAST START`: 最后启动时间 