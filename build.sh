#!/bin/bash

# 设置版本号
VERSION="0.1.0"

# 创建发布目录
mkdir -p release

# 检测当前系统
HOST_OS=$(uname -s)
HOST_ARCH=$(uname -m)

echo "Building version ${VERSION}"
echo "Host system: ${HOST_OS} ${HOST_ARCH}"

# 设置交叉编译环境变量
export CARGO_BUILD_TARGET_DIR="target"
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# Linux 构建
echo "Building for Linux..."
if [ "${HOST_OS}" = "Linux" ]; then
    # 本地构建
    echo "Building native Linux binary..."
    cargo build --release
    cp target/release/hardware_monitor release/hardware_monitor-${VERSION}-linux-${HOST_ARCH}
else
    # 交叉编译
    echo "Cross-compiling for Linux..."
    cargo build --release --target x86_64-unknown-linux-gnu
    cargo build --release --target aarch64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/release/hardware_monitor release/hardware_monitor-${VERSION}-linux-x86_64
    cp target/aarch64-unknown-linux-gnu/release/hardware_monitor release/hardware_monitor-${VERSION}-linux-aarch64
fi

# Windows 构建 (如果在 Linux 上，需要安装 MinGW)
if [ "${HOST_OS}" = "Linux" ]; then
    echo "Cross-compiling for Windows..."
    if command -v x86_64-w64-mingw32-gcc >/dev/null; then
        cargo build --release --target x86_64-pc-windows-gnu
        cp target/x86_64-pc-windows-gnu/release/hardware_monitor.exe release/hardware_monitor-${VERSION}-windows-x86_64.exe
    else
        echo "Skipping Windows build: MinGW not installed"
    fi
elif [ "${HOST_OS}" = "Windows_NT" ]; then
    echo "Building for Windows..."
    cargo build --release
    cp target/release/hardware_monitor.exe release/hardware_monitor-${VERSION}-windows-${HOST_ARCH}.exe
fi

# macOS 构建 (仅在 macOS 上进行)
if [ "${HOST_OS}" = "Darwin" ]; then
    echo "Building for macOS..."
    cargo build --release
    cp target/release/hardware_monitor release/hardware_monitor-${VERSION}-macos-${HOST_ARCH}
fi

# 创建压缩包
echo "Creating archives..."
cd release
for file in *; do
    if [ -f "$file" ]; then
        if [[ "$file" == *.exe ]]; then
            zip "${file%.exe}.zip" "$file"
            rm "$file"
        else
            tar -czf "${file}.tar.gz" "$file"
            rm "$file"
        fi
    fi
done

# 生成 SHA256 校验和
if command -v sha256sum >/dev/null; then
    sha256sum * > SHA256SUMS
elif command -v shasum >/dev/null; then
    shasum -a 256 * > SHA256SUMS
fi

echo "Build complete! Release files are in the release directory."
ls -l
cat SHA256SUMS 