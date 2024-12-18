#!/bin/bash

# 设置版本号
VERSION="0.1.1"

# 创建发布目录
mkdir -p release

# 检测当前系统
HOST_OS=$(uname -s)
HOST_ARCH=$(uname -m)

echo "Building version ${VERSION}"
echo "Host system: ${HOST_OS} ${HOST_ARCH}"

# 安装 cross 工具（如果需要）
if ! command -v cross &> /dev/null; then
    echo "Installing cross..."
    cargo install cross
fi

# 设置目标平台
LINUX_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "armv7-unknown-linux-gnueabihf"
)

WINDOWS_TARGETS=(
    "x86_64-pc-windows-msvc"
    "i686-pc-windows-msvc"
)

MACOS_TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# Linux 构建
echo "Building for Linux targets..."
for target in "${LINUX_TARGETS[@]}"; do
    echo "Building for ${target}..."
    if [ "${target}" = "x86_64-unknown-linux-gnu" ] && [ "${HOST_OS}" = "Linux" ] && [ "${HOST_ARCH}" = "x86_64" ]; then
        # 本地构建
        cargo build --release --target "${target}"
    else
        # 交叉编译
        cross build --release --target "${target}"
    fi
    
    if [ $? -eq 0 ]; then
        cp "target/${target}/release/hardware_monitor" "release/hardware_monitor-${VERSION}-${target}"
        echo "Build successful for ${target}"
    else
        echo "Build failed for ${target}"
    fi
done

# Windows 构建
if [ "${HOST_OS}" = "Windows_NT" ]; then
    echo "Building for Windows targets..."
    for target in "${WINDOWS_TARGETS[@]}"; do
        echo "Building for ${target}..."
        cargo build --release --target "${target}"
        if [ $? -eq 0 ]; then
            cp "target/${target}/release/hardware_monitor.exe" "release/hardware_monitor-${VERSION}-${target}.exe"
            echo "Build successful for ${target}"
        else
            echo "Build failed for ${target}"
        fi
    done
elif command -v x86_64-w64-mingw32-gcc >/dev/null; then
    echo "Cross-compiling for Windows targets..."
    for target in "${WINDOWS_TARGETS[@]}"; do
        echo "Building for ${target}..."
        cross build --release --target "${target}"
        if [ $? -eq 0 ]; then
            cp "target/${target}/release/hardware_monitor.exe" "release/hardware_monitor-${VERSION}-${target}.exe"
            echo "Build successful for ${target}"
        else
            echo "Build failed for ${target}"
        fi
    done
else
    echo "Skipping Windows builds: MinGW not installed and not on Windows"
fi

# macOS 构建
if [ "${HOST_OS}" = "Darwin" ]; then
    echo "Building for macOS targets..."
    for target in "${MACOS_TARGETS[@]}"; do
        echo "Building for ${target}..."
        cargo build --release --target "${target}"
        if [ $? -eq 0 ]; then
            cp "target/${target}/release/hardware_monitor" "release/hardware_monitor-${VERSION}-${target}"
            echo "Build successful for ${target}"
        else
            echo "Build failed for ${target}"
        fi
    done
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