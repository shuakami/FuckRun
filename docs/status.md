# Status 命令

Status命令是FuckRun的"健康管理师"。它不仅能告诉你进程是否在运行，还能提供丰富的状态信息，帮助你全面了解进程的运行状况。

## 命令格式

```bash
fuckrun status [选项]
```

## 核心参数

### 进程名称 (-n, --name)

指定要查看状态的进程名称：

```bash
fuckrun status -n web
```

### Python解释器 (--python)

当检查Python应用时，可以指定Python解释器路径：

```bash
fuckrun status -n web --python /usr/local/bin/python3
```

### 端口号 (-p, --port)

指定要检查的端口号：

```bash
fuckrun status -n web --port 8000
```

## 状态检查

Status命令会执行一系列全面的检查：

1. 进程存活性
   - 检查PID文件
   - 验证进程是否运行
   - 确认进程响应

2. 健康状态
   - 调用健康检查接口
   - 验证服务可用性
   - 检查响应时间

3. 资源使用
   - CPU使用率
   - 内存占用
   - 文件描述符数量

4. 运行信息
   - 启动时间
   - 重启次数
   - 最后活动时间

## 输出信息

Status命令的输出包含多个部分：

### 基本信息

```
进程名称: web
状态: 运行中
PID: 12345
启动时间: 2024-01-13 10:00:00
运行时长: 2小时30分钟
```

### 健康状态

```
健康检查: 通过
响应时间: 200ms
最后检查: 2024-01-13 12:30:00
```

### 资源使用

```
CPU使用率: 2.5%
内存使用: 128MB
虚拟内存: 256MB
文件句柄: 32
```

### 进程树

```
web(12345)
  ├─ worker1(12346)
  └─ worker2(12347)
```

## 平台特性

### Windows平台

在Windows平台上，Status命令会：
- 使用Windows API获取进程信息
- 检查服务状态
- 监控进程树
- 验证monitor进程

### Unix/Linux平台

在Unix/Linux平台上，Status命令会：
- 读取/proc文件系统
- 检查进程信号
- 分析进程组
- 验证会话状态

## 使用场景

### 服务监控

监控Web服务状态：

```bash
fuckrun status -n web --port 5000
```

### 批量检查

检查多个相关服务：

```bash
for service in web api cache; do
    fuckrun status -n $service
done
```

### 健康报告

生成详细的状态报告：

```bash
fuckrun status -n web > status_report.txt
```

## 最佳实践

1. 定期检查服务状态
2. 配置合适的健康检查接口
3. 监控关键性能指标
4. 保存状态检查历史
5. 设置状态告警阈值

## 状态码说明

FuckRun使用标准的状态码来表示进程状态：

- 0: 正常运行
- 1: 已停止
- 2: 正在启动
- 3: 正在停止
- 4: 异常状态
- 5: 状态未知

## 常见问题

### 状态不准确

可能的原因：
- 状态文件过期
- 进程僵死
- 网络延迟
- 权限问题

解决方案：
1. 清理状态缓存
2. 重新启动进程
3. 检查网络连接
4. 验证访问权限

### 健康检查失败

可能的原因：
- 服务未就绪
- 端口未监听
- 配置错误
- 资源不足

解决方案：
1. 等待服务初始化
2. 检查端口配置
3. 验证健康检查URL
4. 增加系统资源

## 进阶功能

### 自定义健康检查

你可以通过配置文件自定义健康检查逻辑：

```yaml
processes:
  web:
    health_check:
      url: http://localhost:5000/health
      interval: 30
      timeout: 5
      retries: 3
      headers:
        User-Agent: FuckRun/1.0
```

### 性能监控

Status命令支持详细的性能监控：

```yaml
processes:
  web:
    monitoring:
      cpu_threshold: 80
      memory_limit: 1024
      disk_usage: true
      network_stats: true
```

### 状态通知

配置状态变更通知：

```yaml
processes:
  web:
    notifications:
      - type: email
        to: admin@example.com
      - type: webhook
        url: http://monitor.example.com/webhook
``` 