# 🎮 Game Asset Studio

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

**2D游戏素材聚合搜索与管理工具**

帮助游戏开发者从多个素材网站搜索、下载和管理2D游戏素材的一站式桌面应用。

---

## ✨ 功能特性

### 🔍 多网站聚合搜索
- 支持 **itch.io**、**OpenGameArt**、**Kenney** 等多个素材网站
- 关键词搜索，一次搜索，多源结果
- 来源筛选，按需选择搜索网站

### 📥 批量下载
- 支持队列下载
- 下载进度显示
- 自动保存到素材库

### 📁 素材库管理
- 本地 SQLite 数据库存储
- 收藏功能
- 已下载筛选
- 离线浏览支持

### 🎨 图像后处理
- 格式转换（PNG/JPG/WebP）
- 尺寸调整
- SpriteSheet 切割
- 项目导出（Unity/Godot/通用）

---

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| **前端** | React 18 + TypeScript + TailwindCSS |
| **构建** | Vite |
| **后端** | Tauri 2 (Rust) |
| **数据库** | SQLite |
| **图标** | Lucide React |

---

## 📦 安装

### 从发布版本安装
1. 从 [Releases](https://github.com/Justinian-A/game-asset-studio/releases) 页面下载最新版本
2. 运行安装程序
3. 按提示完成安装

### 从源码构建

```bash
# 克隆项目
git clone https://github.com/Justinian-A/game-asset-studio.git
cd game-asset-studio

# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建发布版本
npm run tauri build
```

---

## 🚀 使用说明

### 搜索素材
1. 在搜索框输入关键词（如 "pixel art", "character", "tileset"）
2. 选择要搜索的网站（可选）
3. 点击"搜索"按钮
4. 浏览搜索结果

### 下载素材
1. 将鼠标悬停在素材卡片上
2. 点击下载按钮（自动保存到素材库）
3. 在素材库的"已下载"中查看

### 管理素材库
1. 点击左侧"素材库"图标
2. 使用筛选功能：全部 / 收藏 / 已下载
3. 搜索本地素材

---

## 📁 项目结构

```
game-asset-studio/
├── src/                    # React 前端
│   ├── pages/             # 页面组件
│   │   ├── SearchPage.tsx    # 搜索页面
│   │   ├── LibraryPage.tsx   # 素材库页面
│   │   └── SettingsPage.tsx  # 设置页面
│   └── App.tsx            # 主应用
├── src-tauri/             # Rust 后端
│   └── src/
│       ├── lib.rs         # Tauri 命令
│       ├── db.rs          # 数据库模块
│       ├── download.rs    # 下载管理器
│       ├── image.rs       # 图像处理
│       └── search/        # 搜索模块
│           ├── mod.rs     # 搜索引擎
│           ├── itch.rs    # itch.io 客户端
│           ├── oga.rs     # OpenGameArt 客户端
│           └── kenney.rs  # Kenney 客户端
└── docs/                  # 文档
```

---

## 📋 系统要求

- Windows 10/11 (64位)
- 至少 4GB 内存
- 100MB 磁盘空间（不含下载素材）

---

## 📄 许可证

MIT License - 详见 [LICENSE](./LICENSE)

---

## 🙏 致谢

- [itch.io](https://itch.io/) - 独立游戏素材市场
- [OpenGameArt.org](https://opengameart.org/) - 开源游戏素材社区
- [Kenney](https://kenney.nl/) - 免费游戏素材
- [Tauri](https://tauri.app/) - 桌面应用框架
- [React](https://reactjs.org/) - 前端框架
- [TailwindCSS](https://tailwindcss.com/) - CSS 框架

---

## 📧 联系方式

- 项目主页: [GitHub](https://github.com/Justinian-A/game-asset-studio)
- 问题反馈: [Issues](https://github.com/Justinian-A/game-asset-studio/issues)
