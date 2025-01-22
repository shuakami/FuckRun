# FuckRun 健康检查指南 🏥

本文档详细介绍了 FuckRun 的健康检查机制。

## 目录
- [健康检查概述](#健康检查概述)
- [配置选项](#配置选项)
- [工作原理](#工作原理)
- [最佳实践](#最佳实践)
- [故障排查](#故障排查)

## 健康检查概述

健康检查是 FuckRun 用来监控进程状态的重要机制。它通过定期访问进程提供的HTTP接口来确认进程是否正常运行。

主要功能：
- 确保进程正常启动
- 监控进程运行状态
- 支持自动重启
- 提供状态报告

## 配置选项

健康检查相关的配置项：

```yaml
# 进程配置
processes:
  web:
    # 健康检查URL(必填)
    health_check_url: http://localhost:8000/health
    
    # 进程管理配置(可选)
    process:
      # 健康检查超时时间(秒)(默认:5)
      health_check_timeout_secs: 5
      
      # 健康检查重试次数(默认:10)
      health_check_retries: 10
      
      # 重试间隔时间(秒)(默认:2)
      retry_interval_secs: 2
```

### 配置说明

1. `health_check_url`
   - 健康检查的HTTP接口地址
   - 必须返回2xx状态码
   - 建议使用专门的健康检查接口

2. `health_check_timeout_secs`
   - 单次健康检查的超时时间
   - 超过此时间视为检查失败
   - 默认5秒

3. `health_check_retries`
   - 健康检查失败后的重试次数
   - 所有重试都失败才判定为不健康
   - 默认10次

4. `retry_interval_secs`
   - 重试之间的等待时间
   - 避免频繁重试对服务造成压力
   - 默认2秒

## 工作原理

### 启动检查

1. 进程启动后，FuckRun会等待`init_wait_secs`秒
2. 开始执行健康检查：
   - 发送HTTP GET请求到`health_check_url`
   - 等待最多`health_check_timeout_secs`秒
   - 如果超时或失败，进行重试
3. 重试机制：
   - 最多重试`health_check_retries`次
   - 每次重试间隔`retry_interval_secs`秒
   - 全部失败则判定启动失败

### 运行时检查

1. 定期检查进程状态：
   - 检查进程是否存在
   - 访问健康检查接口
2. 异常处理：
   - 检查失败会记录到日志
   - 如果启用了自动重启，会尝试重启进程
3. 状态更新：
   - 检查结果会更新到状态文件
   - 可通过`status`命令查看

## 最佳实践

### 健康检查接口设计

1. 轻量级检查
```python
@app.route('/health')
def health_check():
    return {'status': 'ok'}, 200
```

2. 深度检查
```python
@app.route('/health')
def health_check():
    try:
        # 检查数据库连接
        db.ping()
        
        # 检查缓存服务
        cache.ping()
        
        # 检查消息队列
        queue.ping()
        
        return {
            'status': 'ok',
            'database': 'connected',
            'cache': 'connected',
            'queue': 'connected'
        }, 200
    except Exception as e:
        return {
            'status': 'error',
            'error': str(e)
        }, 500
```

### 配置建议

1. 超时设置
   - `health_check_timeout_secs`: 设置为接口正常响应时间的2-3倍
   - `init_wait_secs`: 根据应用启动时间适当设置

2. 重试策略
   - `health_check_retries`: 建议10-20次
   - `retry_interval_secs`: 建议2-5秒

3. 自动重启
   - 建议开启`auto_restart`
   - 设置合理的`max_restarts`避免无限重启

## 故障排查

### 常见问题

1. 健康检查总是失败
   - 检查URL是否正确
   - 确认端口是否正确
   - 查看应用日志是否有错误
   - 检查防火墙设置
   - **检查是否开启代理导致检查失败（尤其是502 Bad Gateway）**

2. 检查超时
   - 增加`health_check_timeout_secs`
   - 优化健康检查接口性能
   - 检查系统资源使用情况

3. 频繁重启
   - 检查应用日志找出崩溃原因
   - 调整`retry_interval_secs`增加间隔
   - 考虑增加`max_restarts`限制

### 调试方法

1. 启用详细日志
```yaml
global:
  log:
    level: debug
```

2. 手动测试健康检查
```bash
# 使用curl测试
curl http://localhost:8000/health

# 查看进程状态
fuckrun status -n web
```

3. 查看日志
```bash
# 查看进程日志
fuckrun logs -n web

# 查看系统日志
fuckrun logs --system
```