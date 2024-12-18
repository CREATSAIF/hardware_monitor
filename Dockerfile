FROM rust:latest as builder

# 安装交叉编译工具
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64 \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# 添加目标平台
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    x86_64-pc-windows-gnu \
    aarch64-pc-windows-gnu

WORKDIR /usr/src/hardware_monitor
COPY . .

# 设置交叉编译环境变量
ENV CARGO_BUILD_TARGET_DIR=/usr/src/hardware_monitor/target
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# 运行构建脚本
RUN chmod +x build.sh && ./build.sh

# 使用轻量级镜像作为最终镜像
FROM debian:stable-slim
WORKDIR /app
COPY --from=builder /usr/src/hardware_monitor/release ./release
COPY --from=builder /usr/src/hardware_monitor/README.md ./
COPY --from=builder /usr/src/hardware_monitor/LICENSE ./

# 设置入口点
ENTRYPOINT ["./release/hardware_monitor-0.1.0-linux-x86_64"] 