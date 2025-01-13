# List 命令

List命令是FuckRun的"进程总览"工具。它能够帮助你快速了解系统中所有由FuckRun管理的进程状态，提供清晰的进程概览和详细信息。

## 命令格式

```bash
fuckrun list [选项]
```

## 输出格式

List命令的输出经过精心设计，提供了丰富的信息：

### 基础视图

```
名称    状态    PID     端口    运行时间    内存使用
web     运行中  12345   5000    2h30m       128MB
api     运行中  12346   3000    1h15m       256MB
worker  已停止  -       -       -           -
```

### 详细视图

```
Web服务 (web)
  状态: 运行中
  PID: 12345
  端口: 5000
  启动时间: 2024-01-13 10:00:00
  运行时长: 2小时30分钟
  内存使用: 128MB
  健康状态: 正常
  重启次数: 0

API服务 (api)
  状态: 运行中
  PID: 12346
  端口: 3000
  启动时间: 2024-01-13 11:15:00
  运行时长: 1小时15分钟
  内存使用: 256MB
  健康状态: 正常
  重启次数: 1

后台任务 (worker)
  状态: 已停止
  上次运行: 2024-01-13 09:00:00
  退出码: 0
```

## 显示选项

### 输出格式化

List命令支持多种输出格式：

1. 表格格式（默认）
```bash
fuckrun list --format table
```

2. JSON格式
```bash
fuckrun list --format json
```

3. YAML格式
```bash
fuckrun list --format yaml
```

### 信息过滤

可以根据不同条件过滤进程：

```bash
# 按状态过滤
fuckrun list --status running

# 按名称过滤
fuckrun list --name "web*"

# 按端口过滤
fuckrun list --port 5000
```

### 排序选项

支持多种排序方式：

```bash
# 按名称排序
fuckrun list --sort name

# 按运行时间排序
fuckrun list --sort uptime

# 按内存使用排序
fuckrun list --sort memory
```

## 使用场景

### 系统概览

快速查看所有进程状态：

```bash
fuckrun list
```

### 问题诊断

查看异常进程：

```bash
fuckrun list --status error
```

### 资源监控

监控资源使用情况：

```bash
fuckrun list --sort memory --format json > resources.json
```

## 配置集成

List命令可以通过配置文件自定义显示行为：

```yaml
list:
  # 显示配置
  display:
    columns:
      - name
      - status
      - pid
      - port
      - uptime
      - memory
    
    # 默认排序
    sort_by: name
    sort_order: asc
    
    # 默认格式
    format: table
    
  # 过滤配置
  filters:
    - type: status
      value: running
    - type: name
      pattern: "web*"
      
  # 刷新配置
  refresh:
    enabled: true
    interval: 5  # 秒
```

## 高级功能

### 实时监控

启用实时更新模式：

```bash
fuckrun list --watch
```

### 资源统计

显示资源使用统计：

```bash
fuckrun list --stats
```

### 导出功能

支持多种导出格式：

```bash
# 导出为JSON
fuckrun list --export json > processes.json

# 导出为CSV
fuckrun list --export csv > processes.csv

# 导出为HTML报告
fuckrun list --export html > report.html
```

## 最佳实践

1. 监控管理
   - 定期查看进程状态
   - 关注异常进程
   - 监控资源使用
   - 保存状态快照

2. 信息展示
   - 选择合适的输出格式
   - 使用过滤优化显示
   - 关注关键指标
   - 定期导出报告

3. 自动化集成
   - 编写监控脚本
   - 设置告警阈值
   - 自动生成报告
   - 集成CI/CD流程

## 常见问题

### 显示不完整

可能的原因：
- 终端窗口太小
- 字符编码问题
- 进程状态异常
- 权限不足

解决方案：
1. 调整终端大小
2. 检查编码设置
3. 使用详细模式
4. 验证访问权限

### 状态不准确

可能的原因：
- 缓存过期
- 进程状态变化
- 文件系统延迟
- 并发访问

解决方案：
1. 强制刷新显示
2. 使用实时模式
3. 检查状态文件
4. 增加更新频率

## 扩展功能

### 自定义视图

创建自定义显示视图：

```yaml
views:
  minimal:
    columns: [name, status]
    format: compact
    
  detailed:
    columns: [name, status, pid, port, uptime, memory, health]
    format: full
    
  monitoring:
    columns: [name, cpu, memory, disk, network]
    format: table
    refresh: 5
```

### 状态分析

启用智能状态分析：

```yaml
analysis:
  enabled: true
  metrics:
    - type: resource_usage
      threshold: 80
    - type: health_check
      interval: 30
    - type: error_rate
      window: 3600
``` 