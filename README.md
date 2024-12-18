# Hardware Monitor

一个跨平台的硬件监控 API 服务，支持 Linux、Windows 和 macOS。

## 功能特点

- CPU 监控（使用率、频率、温度等）
- GPU 监控（仅支持 NVIDIA，需要安装 NVIDIA 驱动）
- 内存监控（使用率、交换分区等）
- 磁盘监控（使用率、IO 统计等）
- 网络监控（流量、连接状态等）
- 温度监控（各组件温度）
- 电源监控（电池状态、功耗等）
- 进程监控（进程数、状态统计等）
- 系统性能指标

## 系统要求

- Linux: 内核 2.6.32 或更高
- Windows: Windows 7 或更高
- macOS: 10.13 或更高

## 安装

1. 从 release 页面下载适合您系统的版本
2. 解压缩文件
3. 运行可执行文件

## 使用方法

启动服务：
```bash
./hardware_monitor
```

默认端口为 9527，可以通过环境变量修改：
```bash
PORT=8080 ./hardware_monitor
```

## API 端点

- `GET /api/system` - 获取完整的系统信息
- `GET /api/health` - 健康检查
- `GET /api/temperature/history` - 获取温度历史记录

## 构建

需要安装 Rust 工具链和以下依赖：

- Linux: gcc, libssl-dev
- Windows: Visual Studio 构建工具
- macOS: Xcode 命令行工具

构建命令：
```bash
cargo build --release
```

跨平台构建：
```bash
./build.sh
```

## 配置

可以通过环境变量配置：

- `PORT`: 服务端口（默认：9527）
- `RUST_LOG`: 日志级别（默认：info）

## 许可证

MIT License 