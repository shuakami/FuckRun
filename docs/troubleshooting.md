# FuckRun 故障排查指南 🔧

本文档帮助你解决使用 FuckRun 时可能遇到的常见问题。

## 目录
- [常见错误](#常见错误)
- [日志查看](#日志查看)
- [进程管理](#进程管理)
- [配置问题](#配置问题)
- [系统相关](#系统相关)

## 常见错误

### 1. 进程启动失败

#### 症状
- 启动命令返回错误
- 进程状态显示未运行
- 日志中有错误信息

#### 可能原因
1. 程序路径错误
2. 工作目录不存在
3. 权限问题
4. 端口被占用
5. Python解释器问题

#### 解决方法

1. 检查程序路径
```bash
# 确认程序是否存在
ls <program_path>

# 检查Python解释器
python --version
```

2. 检查工作目录
```bash
# 确认目录存在
ls <working_dir>

# 创建目录
mkdir -p <working_dir>
```

3. 检查权限
```bash
# 查看文件权限
ls -l <program_path>

# 修改权限
chmod +x <program_path>
```

4. 检查端口
```bash
# Windows
netstat -ano | findstr <port>

# Linux
netstat -tulpn | grep <port>
```

5. Python相关
```bash
# 检查依赖
pip list

# 安装依赖
pip install -r requirements.txt
```

### 2. 健康检查失败

#### 症状
- 进程启动但健康检查失败
- 日志显示连接超时或拒绝

#### 可能原因
1. 程序未完全启动
2. 健康检查URL配置错误
3. 防火墙阻止
4. 程序内部错误

#### 解决方法

1. 调整等待时间
```yaml
process:
  init_wait_secs: 10
  health_check_timeout_secs: 10
```

2. 验证健康检查
```bash
# 手动测试健康检查
curl http://localhost:<port>/health
```

3. 检查防火墙
```bash
# Windows
netsh advfirewall show currentprofile

# Linux
sudo ufw status
```

4. 健康检查总是失败
   - 检查URL是否正确
   - 确认端口是否正确
   - 查看应用日志是否有错误
   - 检查防火墙设置
   - **检查是否开启代理导致检查失败（尤其是502 Bad Gateway）**

### 3. 自动重启问题

#### 症状
- 进程频繁重启
- 重启次数超过限制
- 无法正常退出

#### 可能原因
1. 程序内存泄漏
2. 配置不当
3. 系统资源不足
4. 依赖服务未就绪

#### 解决方法

1. 调整重启策略
```yaml
processes:
  web:
    auto_restart: true
    max_restarts: 5
    start_delay: 5
```

2. 监控资源使用
```bash
# 查看进程状态
fuckrun list --watch

# 系统资源
top  # Linux
taskmgr  # Windows
```

## 日志查看

### 系统日志

```bash
# 查看FuckRun系统日志
fuckrun logs --system

# 实时跟踪
fuckrun logs --system -f
```

### 进程日志

```bash
# 查看标准输出
fuckrun logs -n <进程名>

# 查看错误日志
fuckrun logs -n <进程名> --type stderr

# 查看特定日期的日志
fuckrun logs -n <进程名> --date 2024-01-22
```

### 日志位置

- 系统日志: `.fuckrun/logs/<日期>/fuckrun.log`
- 进程日志: `.fuckrun/processes/<进程名>/logs/<日期>/{stdout,stderr}.log`

## 进程管理

### 1. 进程状态异常

#### 症状
- 状态显示运行但实际未运行
- PID不匹配
- 无法停止进程

#### 解决方法

1. 重置进程状态
```bash
# 停止进程
fuckrun stop -n <进程名>

# 清理状态文件
rm -f .fuckrun/processes/<进程名>/state.json

# 重新启动
fuckrun start -n <进程名>
```

2. 强制停止
```bash
# Windows
taskkill /F /PID <pid>

# Linux
kill -9 <pid>
```

### 2. 守护进程问题

#### 症状
- 主进程退出后子进程未退出
- 无法正常停止守护进程
- 日志输出异常

#### 解决方法

1. 检查进程树
```bash
# Windows
wmic process where ParentProcessId=<pid> get ProcessId,CommandLine

# Linux
pstree -p <pid>
```

2. 清理僵尸进程
```bash
# 查找相关进程
ps aux | grep <进程名>

# 停止所有相关进程
pkill -f <进程名>
```

## 配置问题

### 1. 配置文件加载失败

#### 症状
- 提示配置文件不存在
- 配置未生效
- 格式错误

#### 解决方法

1. 检查配置文件位置
```bash
# 查看配置文件
cat config.yaml

# 验证YAML语法
python -c "import yaml; yaml.safe_load(open('config.yaml'))"
```

2. 使用默认配置
```bash
# 复制示例配置
cp config.yaml.example config.yaml

# 编辑配置
vim config.yaml
```

### 2. 环境变量问题

#### 症状
- 程序无法访问环境变量
- 环境变量未生效
- 值不正确

#### 解决方法

1. 检查环境变量配置
```yaml
processes:
  web:
    env:
      PORT: 8000
      DEBUG: true
```

2. 验证环境变量
```bash
# 启动时打印环境变量
fuckrun start -n web --env DEBUG=true
```

## 系统相关

### 1. 权限问题

#### Windows特有问题
- 需要管理员权限
- UAC限制
- 防火墙阻止

#### 解决方法
1. 以管理员身份运行
2. 调整UAC设置
3. 添加防火墙规则

#### Linux特有问题
- 文件权限
- SELinux限制
- systemd集成

#### 解决方法
1. 调整文件权限
```bash
chmod +x /path/to/program
chown user:group /path/to/program
```

2. SELinux设置
```bash
# 检查SELinux状态
sestatus

# 允许程序访问网络
semanage port -a -t http_port_t -p tcp <port>
```

### 2. 资源限制

#### 症状
- 内存不足
- CPU使用率高
- 文件描述符耗尽

#### 解决方法

1. 检查系统限制
```bash
# 查看限制
ulimit -a

# 调整限制
ulimit -n 65535  # 文件描述符
```

2. 监控资源使用
```bash
# 实时监控
fuckrun list --watch --filter min-cpu=50

# 查看详细状态
fuckrun status -n <进程名>
```

3. 优化配置
```yaml
process:
  # 调整进程优先级
  priority: normal
  
  # 限制资源使用
  max_memory: 1024m
  max_cpu: 50
``` 