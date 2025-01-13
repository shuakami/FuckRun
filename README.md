# FuckRun

[![Version](https://img.shields.io/badge/version-1.0.0-1E88E5.svg?style=flat-square)]()
[![Build](https://img.shields.io/badge/build-passing-00C853.svg?style=flat-square)]()
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-7E57C2.svg?style=flat-square)]()


FuckRun，如它的名字一样，是一个干脆、简单的进程管理工具。

在开发这个项目之前，我们总是在和各种进程管理的问题作斗争：**复杂晦涩的API设计**、**低效的进程操作**、**恼人的文件系统权限问题**...而现在，这些都不再是问题。

FuckRun的设计理念很简单：**能用底层库就用底层库，需要封装的地方才封装**。我们不重复造轮子，而是专注于提供一个真正好用的工具。

## 🎯 什么是进程管理

在现代操作系统中，进程管理是一个永恒的话题。当你运行一个Web服务，启动一个数据库，或是执行一个长期任务，这些都是进程。**管理好这些进程**，就像照看一群...宝宝

> 你需要知道它们是否健康，并在必要时唤醒它们，也能在合适的时候让它们安静下来。

传统的进程管理往往充斥着各种复杂性。开发者需要编写大量代码来处理进程的启动、监控、重启和关闭。更糟的是，当涉及到文件系统权限、进程间通信这些底层细节时，情况会变得更加棘手。

FuckRun就是为了解决这些痛点而生。它追求**极致的简单**，将进程管理的复杂性隐藏在简洁的接口之下。你不再需要关心底层的实现细节，只需要告诉它"启动这个"、"关闭那个"。它会自动处理好进程的健康检查、优雅关闭、日志管理等所有细节。

## 🚀 快速上手

FuckRun目前处于**1.0.0版本**，暂未发布到crates.io。你可以通过源码构建使用：

```bash
git clone https://github.com/yourusername/fuckrun
cd fuckrun
cargo build --release
```

### 核心命令

FuckRun提供了一组简洁而强大的命令：

```bash
# 启动进程
fuckrun start -n web

# 停止进程
fuckrun stop -n web

# 查看进程状态
fuckrun status -n web

# 查看进程日志
fuckrun logs -n web -f

# 查看系统日志
fuckrun system-logs -f

# 列出所有进程
fuckrun list
```

每个命令都支持丰富的选项，例如：
- 守护进程模式（--daemon）
- 自动重启（--auto-restart）
- 端口指定（--port）
- 环境变量注入（--env）
- 实时日志跟踪（-f/--follow）

## 📂 工作区结构

FuckRun采用清晰的工作区结构，让进程管理更加有序：

```
.
├── app/                    # 应用程序目录
│   └── {process}/         # 进程专属目录
│       ├── app.py         # 应用程序
│       └── config.yaml    # 进程配置
│
├── .fuckrun/              # FuckRun工作目录
│   ├── processes/         # 进程管理目录
│   │   └── {process}/    # 进程状态目录
│   │       ├── state.json # 进程状态
│   │       └── logs/     # 进程日志
│   │           ├── stdout.log
│   │           └── stderr.log
│   └── logs/             # 系统日志目录
│       └── {date}/      # 按日期组织
│           └── fuckrun.log
│
└── config.yaml           # 全局配置文件
```

## ⚙️ 配置系统

FuckRun支持**YAML**和**JSON**格式的配置文件，提供了强大而灵活的配置能力：

```yaml
global:
  # 全局工作目录
  working_dir: .
  # 全局环境变量
  env:
    RUST_LOG: info
  # 日志配置
  log:
    file: logs/app.log
    level: debug
    max_size: 100  # MB
    max_files: 5
  # 进程管理配置
  process:
    init_wait_secs: 3
    health_check_retries: 3
    health_check_timeout: 5

# 进程配置
processes:
  web:
    # 基本配置
    program: python
    args: ["app.py"]
    working_dir: ./app/web
    
    # 运行控制
    auto_restart: true
    start_delay: 0
    max_restarts: 3
    
    # 健康检查
    health_check_url: http://localhost:5000/health
    
    # 环境变量
    env:
      PORT: "5000"
      DEBUG: "true"
    
    # 日志配置
    log:
      level: debug
      max_size: 200
      max_files: 10
```

## 🔧 平台特性

FuckRun在不同平台上提供了优化的实现：

### Windows
- 使用独立的monitor进程
- 支持进程分离（DETACHED_PROCESS）
- 进程组管理
- UTF-8编码支持

### Unix/Linux
- 标准的双fork守护进程
- 会话管理（setsid）
- 文件描述符重定向
- 权限管理（umask）

## 📚 示例

我们在[examples](examples)目录提供了一些常见场景的示例：

- 🌐 Web服务管理
  - Python Flask应用
  - Node.js Express服务
  - Rust Actix应用

- 🔗 多进程协同
  - 主从架构
  - 微服务集群
  - 任务处理池

- 🏥 健康检查
  - HTTP健康检查
  - TCP端口检查
  - 自定义检查脚本

- 📝 日志管理
  - 日志轮转
  - 多进程日志
  - 日志聚合

## 📄 开源协议

本项目采用 **CC BY-NC-SA 4.0** 协议开源