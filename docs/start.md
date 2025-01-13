# Start 命令

启动进程是FuckRun最基础也是最重要的功能。通过Start命令，你可以优雅地启动任何进程，并享受FuckRun提供的全方位进程管理能力。

## 命令格式

```bash
fuckrun start [选项]
```

## 核心参数

### 进程名称 (-n, --name)

指定要启动的进程名称。这个名称将用于：
- 在配置文件中查找对应的进程配置
- 创建进程专属的工作目录和日志目录
- 作为进程的唯一标识符

```bash
fuckrun start -n web
```

### Python解释器 (--python)

当启动Python应用时，可以指定使用的Python解释器路径。这在管理多个Python环境时特别有用：

```bash
fuckrun start -n web --python /usr/local/bin/python3
```

### 端口号 (-p, --port)

为进程指定一个端口号。这在启动Web服务时特别有用：

```bash
fuckrun start -n web --port 8000
```

## 高级选项

### 守护进程模式 (--daemon)

在守护进程模式下启动，这意味着：
- 进程将在后台运行
- 完全脱离终端控制
- 标准输入输出被重定向
- 支持优雅关闭

```bash
fuckrun start -n web --daemon
```

### 自动重启 (--auto-restart)

启用自动重启功能：
- 在进程意外退出时自动重启
- 支持最大重启次数限制
- 记录重启历史
- 智能的重启策略

```bash
fuckrun start -n web --auto-restart
```

### 立即分离 (-d, --detach)

启动后立即分离进程：
- 主进程立即返回
- 不等待初始化完成
- 不执行健康检查
- 适合快速启动场景

```bash
fuckrun start -n web --detach
```

## 配置集成

Start命令与配置系统紧密集成。当指定进程名称时，FuckRun会：

1. 首先在进程目录查找配置：
   ```
   app/{process}/config.yaml
   ```

2. 然后在工作目录查找配置：
   ```
   .fuckrun/processes/{process}/config.yaml
   ```

3. 最后使用全局配置：
   ```
   config.yaml
   ```

配置示例：

```yaml
processes:
  web:
    program: python
    args: ["app.py"]
    working_dir: ./app/web
    auto_restart: true
    health_check_url: http://localhost:5000/health
    env:
      PORT: "5000"
      DEBUG: "true"
```

## 平台特性

### Windows平台

在Windows平台上，Start命令会：
- 使用独立的monitor进程
- 支持进程分离
- 管理进程组
- 自动设置UTF-8编码

### Unix/Linux平台

在Unix/Linux平台上，Start命令会：
- 使用双fork模式
- 创建新会话
- 重定向文件描述符
- 设置umask

## 使用场景

### Web服务启动

启动一个Python Flask应用：

```bash
fuckrun start -n web --python python3 --port 5000 --daemon --auto-restart
```

### 长期任务

启动一个数据处理任务：

```bash
fuckrun start -n worker --daemon
```

### 开发调试

快速启动并查看输出：

```bash
fuckrun start -n dev --port 3000
```

## 最佳实践

1. 总是为进程指定一个有意义的名称
2. 在生产环境使用守护进程模式
3. 为重要服务启用自动重启
4. 配置健康检查URL
5. 使用环境变量进行配置注入
6. 合理设置工作目录
7. 注意日志配置

## 常见问题

### 进程启动失败

可能的原因：
- 程序路径不正确
- 权限不足
- 端口被占用
- 依赖缺失

解决方案：
1. 检查配置文件
2. 查看错误日志
3. 确认权限设置
4. 验证依赖安装

### 健康检查超时

可能的原因：
- 初始化时间过长
- 网络问题
- 服务未就绪

解决方案：
1. 增加初始化等待时间
2. 调整健康检查超时设置
3. 验证健康检查URL 