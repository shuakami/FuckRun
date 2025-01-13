# Logs 命令

日志是了解进程行为的窗口。Logs命令提供了强大的日志查看和分析能力，帮助你实时监控进程输出，排查问题，分析性能。

## 命令格式

```bash
fuckrun logs [选项]
```

## 核心参数

### 进程名称 (-n, --name)

指定要查看日志的进程名称：

```bash
fuckrun logs -n web
```

### 实时跟踪 (-f, --follow)

实时查看日志输出：

```bash
fuckrun logs -n web -f
```


## 日志系统

FuckRun的日志系统设计精良，具有以下特点：

### 日志分类

1. 进程日志
   - 标准输出(stdout.log)
   - 标准错误(stderr.log)
   - 应用日志(app.log)

2. 系统日志
   - FuckRun运行日志
   - 监控日志
   - 审计日志

### 日志格式

标准日志格式包含：
```
[时间戳] [日志级别] [进程名称] [文件:行号] - 日志内容
```

例如：
```
[2024-01-13 12:00:00] [INFO] [web] [app.py:42] - 服务启动成功
```

### 日志轮转

FuckRun支持智能的日志轮转：
- 按大小轮转
- 按时间轮转
- 自动压缩
- 保留策略

## 日志目录结构

```
.fuckrun/
├── processes/
│   └── web/
│       └── logs/
│           ├── 2024-01-13/
│           │   ├── stdout.log
│           │   └── stderr.log
│           └── 2024-01-12/
│               ├── stdout.log
│               └── stderr.log
└── logs/
    └── 2024-01-13/
        └── fuckrun.log
```


## 日志配置

可以通过配置文件自定义日志行为：

```yaml
processes:
  web:
    log:
      # 日志文件配置
      file: logs/web.log
      level: debug
      
      # 轮转配置
      max_size: 100    # MB
      max_files: 5     # 保留文件数
      
      # 格式配置
      format: "[{time}] [{level}] {message}"
      time_format: "%Y-%m-%d %H:%M:%S"
      
      # 过滤配置
      filters:
        - level: error
          pattern: ".*Exception.*"
        - level: warn
          pattern: ".*Warning.*"
```

## 最佳实践

1. 日志管理
   - 合理设置日志级别
   - 配置日志轮转
   - 定期清理旧日志
   - 备份重要日志

2. 日志内容
   - 记录关键操作
   - 包含上下文信息
   - 使用合适的日志级别
   - 避免敏感信息

3. 日志分析
   - 定期检查错误日志
   - 监控异常模式
   - 建立日志基线
   - 设置告警阈值

## 高级功能

### 日志聚合

配置日志聚合功能：

```yaml
logging:
  aggregation:
    enabled: true
    server: logs.example.com
    port: 5000
    protocol: tcp
```

### 日志过滤

设置日志过滤规则：

```yaml
logging:
  filters:
    - exclude: ".*DEBUG.*"
    - include: ".*ERROR.*"
    - pattern: "^\\[\\d{4}-\\d{2}-\\d{2}\\]"
```

### 日志分析

启用日志分析功能：

```yaml
logging:
  analysis:
    pattern_recognition: true
    error_detection: true
    performance_tracking: true
```
