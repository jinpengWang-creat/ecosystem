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

## 数据转换 serde 生态

- serde 介绍
- serde 背后发生了什么
- serde-json / serde-yaml / serde-toml / bincode / ...
- serde-with
- serde 使用的注意事项

## 异步运行时 tokio 生态

- tokio
- bytes
- prost
- tokio-stream
- tokio-utils
- loom
- axum

## Tower/Hyper 生态

- tower Service trait: 如何组合服务功能
- tower utility services
  - rate limit
  - load balance
  - ...
- tower-http: HTTP 相关的 service
- Hyper: web 客户端和服务端协议
  - reqwest
  - warp
  - axum
  - tonic

## 数据库处理

- ORM:
  - diesel
  - seaORM
- Sql toolkits: sqlx
  - FromRow
  - Row
- 为什么我们要避免使用 ORM
  - 性能
  - 不太需要的额外抽象
  - 过于中庸，限制太多
    - insert into on conflict ...
    - CTE
  - sql injection 已经收到足够重视
  - 语言绑定，平台绑定
- 构建搞笑且复杂的 SQL 是每个工程师的基本功
- 构建一个 url shortener
  - tokio
  - axum
  - sqlx
  - nanoid

### 如何高效利用 Rust 社区信息给自己提供帮助

- 资源
  - crates.io
  - lib.rs
  - docs.rs
  - reddit.com/r/rust
  - github.com/trending/rust
  - youtube.com/@jonhoo
  - rustcc.cn
- 路径
  - 发现好的 crate
  - 简单阅读 readme / docs.rs
  - clone 到本地
  - 查阅和运行 examples
  - 学习 examples
  - 从 examples 跳转到感兴趣的代码阅读
