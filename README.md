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

启动一个进程就是这么简单：

```bash
fuckrun start -n web
```

## ⚙️ 配置说明

FuckRun支持**YAML**配置文件，这是一个典型的配置示例：

```yaml
global:
  working_dir: .
  env:
    RUST_LOG: info
  log:
    file: logs/app.log
    level: debug

processes:
  web:
    program: python
    args: ["app.py"]
    auto_restart: true
    health_check_url: http://localhost:5000/health
```

所有配置项都提供了**合理的默认值**，你只需要关注你关心的配置即可。

## 📚 示例

我们在[examples](examples)目录提供了一些常见场景的示例：

- 🌐 简单的Web服务管理
- 🔗 多进程协同工作
- 🏥 自定义健康检查
- 📝 日志管理配置

## 📄 开源协议

本项目采用 **CC BY-NC-SA 4.0** 协议开源