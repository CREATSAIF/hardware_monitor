#!/bin/bash

# 检查版本号参数
if [ -z "$1" ]; then
    echo "请提供版本号，例如: ./release.sh 1.0.0"
    exit 1
fi

VERSION=$1

# 更新 Cargo.toml 中的版本号
sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# 提交更改
git add Cargo.toml
git commit -m "release: version $VERSION"

# 创建标签
git tag -a "v$VERSION" -m "Release version $VERSION"

# 推送到远程仓库
git push origin main
git push origin "v$VERSION"

echo "版本 $VERSION 发布流程已启动"
echo "请在 GitHub 上查看 Actions 进度：https://github.com/你的用户名/hardware_monitor/actions" 