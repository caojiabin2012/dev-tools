# 开发者工具箱 (Tool Kit)

一个基于 React + TypeScript + Vite + Tauri 构建的桌面开发者工具应用。

## 项目简介

这是一个轻量级的开发者工具箱桌面应用，提供常用的开发辅助工具，帮助提升日常开发效率。

## 技术栈

- **前端框架**: React 18 + TypeScript
- **构建工具**: Vite
- **桌面框架**: Tauri (Rust 后端)
- **语言**: 中文 (Chinese)

## 功能特性

应用目前包含以下开发工具：

### JSON 格式化工具
- 支持 JSON 数据的格式化与美化
- 可自定义缩进大小 (2/4 空格)
- 数据验证与语法检查

### 侧边栏导航
- 简洁的工具列表导航
- 快速切换不同工具

## 项目结构

```
├── src/                      # React 前端源码
│   ├── components/           # UI 组件
│   │   ├── json-formatter.tsx  # JSON 格式化组件
│   │   └── sidebar.tsx        # 侧边栏组件
│   ├── App.tsx              # 主应用组件
│   ├── main.tsx             # 入口文件
│   └── index.css           # 全局样式
├── src-tauri/               # Tauri Rust 后端
│   ├── src/
│   │   ├── lib.rs           # 库入口
│   │   └── main.rs          # 主程序
│   ├── Cargo.toml           # Rust 依赖配置
│   └── tauri.conf.json      # Tauri 配置
├── public/                 # 静态资源
└── package.json           # Node 依赖配置
```

## 快速开始

### 环境要求

- Node.js 18+
- pnpm (推荐) 或 npm/yarn
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
# 启动前端开发服务器
pnpm dev

# 或启动 Tauri 开发模式
pnpm tauri dev
```

### 构建应用

```bash
# 构建生产版本
pnpm tauri build
```

构建完成后，安装包将生成在 `src-tauri/target/release/bundle` 目录下。

## 使用说明

1. 启动应用后，在侧边栏选择需要的工具
2. JSON 格式化工具：粘贴 JSON 文本，点击格式化按钮即可美化输出
3. 支持深色/浅色主题切换

## 许可证

MIT License

## 贡献指南

欢迎提交 Issue 和 Pull Request！