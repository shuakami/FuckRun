# FuckRun 🚀

[![English](https://img.shields.io/badge/English-red.svg?style=flat-square)](./README.md)
[![版本](https://img.shields.io/badge/version-1.0.0-FBC02D.svg?style=flat-square)]()
[![构建状态](https://img.shields.io/badge/build-passing-FF9800.svg?style=flat-square)]()
[![平台](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-2196F3.svg?style=flat-square)]()

> 欢迎进入高效程序管理的世界！**FuckRun**是你的得力助手——它帮你管理程序、记录日志，并确保程序平稳运行。

## 为什么需要 FuckRun？

随着项目规模的扩大以及运行程序数量的增加，你可能会面临以下问题：
- 必须打开多个终端窗口运行不同的程序
- 难以找到程序的日志
- 程序突然崩溃，没收到任何提示
- 需要手动重启程序

**FuckRun**可以帮助你解决这些问题，它像一个贴心的管家，能：
- 管理所有运行中的程序
- 自动记录日志
- 程序出现问题时及时通知你
- 遇到崩溃时自动重启

## 统一的输出格式 

为了便于与其他程序（如 Python）进行集成，**FuckRun**使用简单一致的输出格式，方便解析和处理数据。

### 输出格式说明

所有状态信息都遵循以下格式：
```
STATUS:ACTION:NAME[:PID]
```

其中：
- `STATUS`：固定前缀，表示这是一个状态信息。
- `ACTION`：描述执行的动作，可能的值包括：
  - `STARTING`：程序正在启动
  - `STARTED`：程序已启动（会包含PID）
  - `STOPPING`：程序正在停止
  - `STOPPED`：程序已停止
  - `RUNNING`：程序正在运行（会包含PID）
- `NAME`：程序名称。
- `PID`：进程ID（仅在`STARTED`和`RUNNING`状态下存在）。

### Python 解析示例

```python
def parse_fuckrun_status(line):
    """解析 FuckRun 的状态输出"""
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

# 使用示例
import subprocess

# 启动程序
process = subprocess.Popen(
    ["fuckrun", "start", "-n", "web"], 
    stdout=subprocess.PIPE,
    text=True
)

# 读取输出
for line in process.stdout:
    status = parse_fuckrun_status(line)
    if status:
        print(f"程序 {status['name']} {status['action']}")
        if status['pid']:
            print(f"PID: {status['pid']}")
```

### 示例状态输出

```bash
# 启动程序时的输出
STATUS:STARTING:web
STATUS:STARTED:web:1234

# 查看程序状态时的输出
STATUS:RUNNING:web:1234

# 停止程序时的输出
STATUS:STOPPING:web
STATUS:STOPPED:web

# 如果遇到错误导致程序终止
Error: [错误信息] 
```

## 开始使用 - 5分钟快速入门 🏃

> ⚠️ 在开始之前：
> 1. 确保已安装[Git](https://git-scm.com/)
> 2. 避免使用包含中文或空格的路径
> 3. 如果遇到权限问题，请尝试以管理员身份运行

### 第一步：安装 FuckRun

打开终端（Windows系统按 Win+R 并输入 `cmd`），运行以下命令：

```bash
# 1. 克隆仓库
git clone https://github.com/yourusername/fuckrun

# 2. 进入目录
cd fuckrun

# 3. 编译安装
cargo build --release
```

> 💡 如果编译过程较慢，可以[下载预编译版本](https://github.com/yourusername/fuckrun/releases)

### 第二步：创建配置文件

在你的项目目录下创建一个 `config.yaml` 文件：

```yaml
# config.yaml
processes:
  # 'web' 是程序的名称，可以修改
  web:
    # 使用 Python 运行 app.py
    program: python
    args: ["app.py"]
    # 程序的工作目录
    working_dir: ./app
    # 程序崩溃后自动重启
    auto_restart: true
```

> 💡 更多高级配置选项将在后续部分详细介绍。

### 第三步：启动你的程序

```bash
# 启动名为 'web' 的程序
fuckrun start -n web
```

完成！现在，**FuckRun**会帮你管理这个程序。

想要查看程序状态吗？试试这些命令：
```bash
# 查看程序状态
fuckrun status -n web

# 查看程序日志
fuckrun logs -n web -f
```

## 下一步做什么？

- 👉 [了解所有命令](docs/commands.md)
- 👉 [完整的配置文件说明](docs/config.md)
- 👉 [健康检查配置教程](docs/health-check.md)
- 👉 [常见问题排查](docs/troubleshooting.md)

## 理解 FuckRun 的工作原理 🔍

> 了解 **FuckRun** 的工作原理，能够帮助你更好地使用它。

**FuckRun** 的核心思想很简单：它像一个管家，帮你照看所有程序。具体来说：

1. **启动程序**：当你让 **FuckRun** 启动一个程序时：
   - 它会创建一个独立的进程来运行你的程序
   - 收集程序输出到日志文件
   - 记录程序状态（PID、启动时间等）

2. **监控运行**：程序运行过程中，**FuckRun** 会：
   - 定期检查程序是否仍在运行
   - 记录程序输出到日志文件
   - 发现异常时立即通知你

3. **自动恢复**：如果程序出现问题，**FuckRun** 会：
   - 记录错误原因
   - 尝试重启程序
   - 通知你发生了什么

下面是简单的流程概览：
```ascii
[你的程序] <─────┐
                │
                ▼
          [FuckRun 管家]
                │
       ┌────────┴───────┐
       ▼              ▼
  [照看程序]       [记录日志]
  - 启动/停止     - 输出日志
  - 健康检查      - 错误日志
  - 自动重启      - 系统日志
```

## 常用命令指南 ⌨️

> 这一节将教你如何使用 **FuckRun** 最常用的命令，每个命令配有示例，跟着做试试看！

### 启动程序

```bash
# 最基础的启动命令
fuckrun start -n web

# 启动并查看日志
fuckrun start -n web --tail

# 启动多个程序
fuckrun start -n web -n api -n worker

# 指定应用程序目录
fuckrun start -n web --app-dir ./deployments/web/app

# 启动并自动重启，同时指定目录
fuckrun start -n web --app-dir ./deployments/web/app --daemon --auto-restart
```

### 停止程序

```bash
# 正常停止
fuckrun stop -n web

# 强制停止（不建议，除非程序卡死）
fuckrun stop -n web --force

# 停止所有程序
fuckrun stop --all
```

### 查看状态

```bash
# 查看单个程序状态
fuckrun status -n web

# 查看所有程序状态
fuckrun status --all

# 输出 JSON 格式的状态（方便程序处理）
fuckrun status -n web --json
```

### 查看日志

```bash
# 查看最新日志
fuckrun logs -n web

# 实时查看日志
fuckrun logs -n web -f

# 仅查看错误日志
fuckrun logs -n web --error

# 查看最近 100 行日志
fuckrun logs -n web -n 100
```

> 💡 加上 `-f` 参数可以实时查看日志，就像 `tail -f` 一样。

## 项目结构说明 📁

> 这一节帮助你了解 **FuckRun** 的文件组织结构，方便你查找文件。

**FuckRun** 会在你的项目中创建如下目录结构：

```
你的项目目录/
├── deployments/             # 应用程序部署目录
│   └── {程序名}/           # 每个程序一个目录
│       └── app/            # 应用程序目录
│           ├── app.py      # 程序本体
│           └── config.yaml # 程序配置文件
│
├── .fuckrun/              # FuckRun 的工作目录
│   ├── processes/         # 程序运行信息
│   │   └── {程序名}/     # 每个程序的信息
│   │       ├── state.json # 运行状态
│   │       └── logs/     # 日志目录
│   │           ├── stdout.log  # 标准输出
│   │           └── stderr.log  # 错误输出
│   └── logs/             # FuckRun 自身日志
│       └── {日期}/      # 按日期分类
│           └── fuckrun.log
└── config.yaml           # 全局配置文件
```

> ⚠️ 注意事项：
> 1. 不要手动修改 `.fuckrun` 目录里的文件
> 2. 日志文件会自动分割，不用担心文件过大
> 3. 配置文件修改后需要重启程序才能生效
> 4. 可以通过 `--app-dir` 参数自定义应用程序目录位置

