# Game Asset Studio

2D游戏素材聚合搜索与管理工具

## 简介

Game Asset Studio 是一款桌面端工具，帮助游戏开发者从多个素材网站（itch.io、OpenGameArt、Kenney）搜索、下载和管理2D游戏素材。

## 功能特性

### 🔍 多网站聚合搜索
- 支持 itch.io、OpenGameArt、Kenney 等多个素材网站
- 关键词搜索，一次搜索，多源结果
- 来源筛选，按需选择搜索网站

### 📥 批量下载
- 支持队列下载
- 下载进度显示
- 断点续传（计划中）

### 📁 素材库管理
- 本地数据库存储
- 收藏功能
- 标签分类
- 离线浏览支持

### 🎨 图像后处理
- 格式转换（PNG/JPG/WebP）
- 尺寸调整
- SpriteSheet切割
- 项目导出（Unity/Godot/通用）

## 技术栈

- **前端**: React 18 + TypeScript + TailwindCSS + Vite
- **后端**: Tauri 2 (Rust)
- **数据库**: SQLite
- **图标**: Lucide React

## 系统要求

- Windows 10/11 (64位)
- 至少 4GB 内存
- 100MB 磁盘空间（不含下载素材）

## 安装

### 从发布版本安装
1. 从 Releases 页面下载最新版本
2. 运行安装程序
3. 按提示完成安装

### 从源码构建
```bash
# 克隆项目
git clone <repo-url>
cd game-asset-studio

# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建发布版本
npm run tauri build
```

## 使用说明

### 搜索素材
1. 在搜索框输入关键词
2. 选择要搜索的网站（可选）
3. 点击"搜索"按钮
4. 浏览搜索结果

### 下载素材
1. 将鼠标悬停在素材卡片上
2. 点击下载按钮
3. 在下载列表查看进度

### 管理素材库
1. 点击"保存到素材库"按钮收藏素材
2. 在素材库页面查看已保存的素材
3. 使用筛选功能快速找到需要的素材

### 导出素材
1. 在设置页面配置导出格式
2. 选择要导出的素材
3. 点击导出按钮

## 目录结构

```
D:\GameAssetStudio\
├── dev\                    # 开发目录
│   └── game-asset-studio\  # 项目源码
├── downloads\              # 下载的素材
└── exports\                # 导出的素材
```

## 配置文件

配置文件位置：`%APPDATA%\com.game-asset-studio.app\`

## 开发日志

详见 [开发日志](./docs/dev-logs/)

## 许可证

MIT License

## 联系方式

- 项目主页: [GitHub](https://github.com/your-username/game-asset-studio)
- 问题反馈: [Issues](https://github.com/your-username/game-asset-studio/issues)

## 致谢

- [itch.io](https://itch.io/) - 独立游戏素材市场
- [OpenGameArt.org](https://opengameart.org/) - 开源游戏素材社区
- [Kenney](https://kenney.nl/) - 免费游戏素材
- [Tauri](https://tauri.app/) - 桌面应用框架
- [React](https://reactjs.org/) - 前端框架
- [TailwindCSS](https://tailwindcss.com/) - CSS 框架
