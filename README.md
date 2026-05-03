<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/FINAL_MosaPlay.png">
    <img src="assets/FINAL_MosaPlay.png" width="128" alt="MosaPlay">
  </picture>
</p>

<h1 align="center">MosaPlay</h1>
<p align="center"><strong>多宫格视频播放器 · Multi-Video Grid Player</strong></p>
<p align="center">一个窗口同时播放多个视频，独立控制每一个。<br>Play multiple videos in one window. Control each independently.</p>

<p align="center">
  <a href="#-中文">🇨🇳 中文</a> &nbsp;|&nbsp;
  <a href="#-english">🇺🇸 English</a>
</p>

---

# 🇨🇳 中文

## 简介

MosaPlay 是一个极简、高性能的多视频同屏播放器。基于 Rust + libmpv + OpenGL 构建，支持 macOS 和 Windows。

无论视频对比、监控回放、多机位剪辑参考，还是单纯想在一个屏幕上同时看多个视频——MosaPlay 都能胜任。

## 特性

- **一个窗口，多个视频** — 自动宫格排列，支持 16+ 路同时播放
- **独立控制** — 每个视频独立控制播放/暂停/进度/速度/音量/字幕
- **鼠标悬停即控** — 鼠标移到哪个视频，键盘快捷键就控制哪个
- **智能布局** — 自动适配视频宽高比，横竖混排时动态分栏，最大化填充、最小化黑边
- **拖放操作** — 拖视频到格子=替换，拖到边距=新增，拖字幕文件=外挂字幕
- **GPU 硬件加速** — Metal (macOS) / D3D11 (Windows)，低 CPU 占用
- **字幕支持** — ASS/SSA/SRT 全格式，libass 渲染
- **轻量** — 二进制 ~2MB，免安装

## 下载

| 平台 | 下载 | 说明 |
|------|------|------|
| **macOS** | [MosaPlay-macOS.zip](https://github.com/dondmq-OC/mosaplay/releases/tag/v0.1.0) | 解压运行或拖入 /Applications |
| **Windows** | [MosaPlay-win64.zip](https://github.com/dondmq-OC/mosaplay/releases/tag/v0.1.0) | 解压即用，无需安装 |

> 最新版在 [Releases 页面](https://github.com/dondmq-OC/mosaplay/releases)。  
> Windows 版由 GitHub Actions 自动构建。

## 快捷键

| 键 | 功能 |
|---|---|
| `Space` | 播放/暂停（焦点视频） |
| `←` `→` | 后退/前进 5s（Ctrl: 30s） |
| `↑` `↓` | 加速/减速 0.25x |
| `[` `]` | 音量 -5/+5 |
| `M` | 静音 |
| `S` | 字幕 开/关 |
| `R` | 重置速度 1x |
| `Tab` | 切换焦点 |
| `1`–`9`, `0` | 选择第 1–10 个视频 |
| `Delete` | 关闭焦点视频 |
| `G` | 刷新布局 |
| `F` | 全屏 |
| `Esc` | 退出 |

### 鼠标

| 操作 | 效果 |
|---|---|
| 悬停 | 自动聚焦 |
| 中键点击 | 关闭视频 |
| 拖视频到格子 | 替换 |
| 拖视频到边距 | 新增 |
| 拖字幕 (.srt/.ass) | 外挂字幕 |

## 从源码构建

```bash
# macOS
brew install mpv sdl2
cargo build --release
# → target/release/mosaplay

# .app 打包
cp target/release/mosaplay target/MosaPlay.app/Contents/MacOS/
```

## 征集意见

MosaPlay 正在积极开发中。欢迎通过以下方式参与：

- 🐛 [提交 Bug](https://github.com/dondmq-OC/mosaplay/issues/new?labels=bug)
- 💡 [功能建议](https://github.com/dondmq-OC/mosaplay/issues/new?labels=enhancement)
- 🗳️ [查看已有需求 & 投票](https://github.com/dondmq-OC/mosaplay/issues)
- 💬 [Discussion 讨论区](https://github.com/dondmq-OC/mosaplay/discussions)

你的每条反馈都会影响 MosaPlay 的发展方向。

---

# 🇺🇸 English

## About

MosaPlay is a minimalist, high-performance multi-video grid player. Built with Rust + libmpv + OpenGL, available for macOS and Windows.

## Features

- **Multiple videos in one window** — Auto grid layout, 16+ simultaneous streams
- **Independent per-video control** — Play/pause/seek/speed/volume/subtitles
- **Hover-to-focus** — Move your cursor to control any video
- **Smart layout** — Auto-adapts to aspect ratios, split layout for mixed portrait/landscape
- **Drag & drop** — Drop on cell to replace, drop on edge to add, drop subtitles to load
- **GPU accelerated** — Metal (macOS) / D3D11 (Windows)
- **Subtitle support** — ASS/SSA/SRT via libass
- **Lightweight** — ~2MB binary, portable

## Download

| Platform | Download | Notes |
|----------|----------|-------|
| **macOS** | [MosaPlay-macOS.zip](https://github.com/dondmq-OC/mosaplay/releases/tag/v0.1.0) | Unzip & run, or drag to /Applications |
| **Windows** | [MosaPlay-win64.zip](https://github.com/dondmq-OC/mosaplay/releases/tag/v0.1.0) | Unzip & run, no install needed |

## Shortcuts

See Chinese section above for full keyboard/mouse reference.

## Feedback

MosaPlay is under active development. We welcome your input:

- 🐛 [Report a Bug](https://github.com/dondmq-OC/mosaplay/issues/new?labels=bug)
- 💡 [Suggest a Feature](https://github.com/dondmq-OC/mosaplay/issues/new?labels=enhancement)
- 🗳️ [Vote on Roadmap](https://github.com/dondmq-OC/mosaplay/issues)
- 💬 [Discussions](https://github.com/dondmq-OC/mosaplay/discussions)

## Build

```bash
# macOS
brew install mpv sdl2
cargo build --release
```

Windows builds via GitHub Actions (see `.github/workflows/build-windows.yml`).

---

<p align="center">
  <sub>Built with Rust · libmpv · SDL2 · OpenGL</sub>
</p>
