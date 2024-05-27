# 第四周：ecosystem

Rust 生态系统

## 错误处理

- anyhow: 统一，简单的错误处理，适用于应用程序级别
- thiserror: 自定义，丰富的错误处理，适用于库级别
- snafu: 更细粒度地管理错误

## 日志处理

- tracing: 记录各种日志
- tracing-subscriber: 输出日志
- open-telemetry\*: 和 open-telemetry 生态互动

## 宏

- derive_builder: 构建数据结构的 builder
- derive_more: 标准库 trait 的自动实现
- strum: enum 相关的 trait 的自动实现
