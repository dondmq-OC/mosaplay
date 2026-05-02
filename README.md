# GridPlayer

> 多宫格视频播放器 — 一个窗口同时播放 1~16+ 个视频，独立控制每一个。

<p align="center">
  <img src="assets/icon_256.png" width="128" alt="GridPlayer icon">
</p>

GridPlayer 是一个极简、高性能的多视频同屏播放器。基于 Rust + libmpv + OpenGL 构建，支持 macOS 和 Windows。无论是视频对比、监控回放、多机位剪辑参考，还是单纯想在一个屏幕上同时看多个视频——GridPlayer 都能胜任。

## 特性

- **一个窗口，多个视频** — 自动宫格排列（2×2, 3×3, 4×4...），支持 16+ 路同时播放
- **独立控制** — 每个视频独立控制播放/暂停/进度/速度/音量
- **鼠标悬停即控** — 鼠标移到哪个视频，键盘快捷键就控制哪个
- **智能布局** — 自动适配视频宽高比，横屏竖屏混排时动态分栏，最大化填充面积、最小化黑边
- **拖放操作** — 拖视频文件到格子=替换，拖到窗口边缘=新增，拖字幕文件=外挂字幕
- **GPU 硬件加速** — Metal (macOS) / D3D11 (Windows)，低 CPU 占用
- **极简界面** — 无边框、无按钮、无菜单，纯键盘+鼠标操作
- **字幕支持** — ASS/SSA/SRT 全格式，libass 渲染，拖入即加载
- **轻量** — 二进制 ~2MB，内存占用低
- **跨平台** — macOS (Apple Silicon/Intel) + Windows (x64)，免安装便携版

## 下载

| 平台 | 下载 | 说明 |
|------|------|------|
| **macOS** | [GridPlayer.app](https://github.com/dondmq-OC/gridplayer/releases) | 拖入 /Applications |
| **Windows** | [GridPlayer-win64.zip](https://github.com/dondmq-OC/gridplayer/releases) | 解压即用，无需安装 |

> 最新版本在 [GitHub Releases](https://github.com/dondmq-OC/gridplayer/releases) 页面。
> Windows 版本通过 GitHub Actions 自动构建，每次 push 都会更新。

## 快捷键

| 键 | 功能 |
|---|---|
| `Space` | 播放 / 暂停（当前焦点视频） |
| `←` `→` | 后退 / 前进 5 秒（Ctrl: 30 秒） |
| `↑` `↓` | 加速 / 减速 0.25x |
| `[` `]` | 音量 -5 / +5 |
| `M` | 静音切换 |
| `S` | 字幕 显示 / 隐藏 |
| `R` | 重置播放速度为 1x |
| `Tab` / `Shift+Tab` | 切换到下一个 / 上一个视频 |
| `1`–`9`, `0` | 直接选择第 1–10 个视频 |
| `Delete` | 关闭当前焦点视频 |
| `G` | 刷新布局（重读视频宽高比） |
| `F` | 全屏切换 |
| `Esc` / `Cmd+Q` | 退出 |

### 鼠标操作

| 操作 | 效果 |
|---|---|
| 鼠标移动到视频上 | 自动切换焦点 |
| 鼠标中键点击视频 | 关闭该视频 |
| 拖视频文件到某格子 | 替换该格子 |
| 拖视频文件到窗口边缘 | 新增视频 |
| 拖字幕文件 (.srt/.ass) 到窗口 | 为焦点视频加载外挂字幕 |

## 使用方法

### macOS

```bash
# 命令行启动
./gridplayer video1.mp4 video2.mp4 video3.mp4

# 或双击 GridPlayer.app，在文件对话框中选择视频（Cmd+点击多选）
open GridPlayer.app
```

### Windows

```cmd
# 命令行启动
gridplayer.exe video1.mp4 video2.mp4

# 或双击 gridplayer.exe，选择视频文件
```

## 从源码构建

### 依赖

- **Rust** 1.70+
- **libmpv** (macOS: `brew install mpv`, Windows: 自动下载)
- **SDL2** (macOS: `brew install sdl2`, Windows: vcpkg)

### 编译

```bash
# macOS
cargo build --release
# 输出: target/release/gridplayer

# .app 打包
cp target/release/gridplayer target/GridPlayer.app/Contents/MacOS/
```

Windows 构建由 GitHub Actions 自动完成，参见 [.github/workflows/build-windows.yml](.github/workflows/build-windows.yml)。

## 技术架构

```
SDL2 → OpenGL 3.3 → 每个 mpv 实例渲染到独立 FBO → 合成到屏幕宫格

Rust 1.95  |  SDL2 2.32  |  OpenGL 3.3  |  libmpv 0.41 (FFmpeg 8.1)
macOS: Metal  |  Windows: D3D11  |  硬件解码: auto-safe
```

- **渲染管线**: 每个视频一个 `mpv_render_context`，渲染到独立的 OpenGL FBO，主循环将所有 FBO 纹理合成到屏幕
- **布局引擎**: 遍历所有可行宫格 + 分栏布局，选填充率最高的
- **输入**: SDL2 事件 + 鼠标位置追踪实现无界面操作
- **字幕**: mpv 内置 libass，支持完整 ASS/SSA 样式

## 许可证

MIT License

---

<p align="center">
  <sub>Built with Rust · libmpv · SDL2 · OpenGL</sub>
</p>
