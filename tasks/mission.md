# Mission Status

> 最后更新: 2026-05-02 重启前 checkpoint

## 项目：GridPlayer — 多宫格视频播放器
- **GitHub**: https://github.com/dondmq-OC/gridplayer (8 commits)
- **本地 .app**: `target/GridPlayer.app` (1.9MB binary, 7.6MB bundle)

## 已完成
- [x] 技术调研：libmpv 是最优播放器内核
- [x] 架构：Rust + SDL2 + OpenGL 3.3 + libmpv render API
- [x] 多实例 mpv 引擎：FBO 渲染，共享 OpenGL context
- [x] GPU 硬件加速：Metal (macOS), hwdec=auto-safe
- [x] Grid 布局：auto / 2×2 / 3×3 / 4×3 / 4×4, G 键切换
- [x] 键盘快捷键：播放/暂停/Seek/速度/音量/焦点/全屏
- [x] macOS .app 打包 + 应用图标（重影播放按钮）
- [x] **vo=libmpv 修复**：防止每个视频弹独立窗口
- [x] 文件对话框：中文标题，多选
- [x] Karpathy 四项原则写入 CLAUDE.md
- [x] 后台任务清理：无残留进程

## 重要技术细节（重启后必读）
- `vo=libmpv` 是关键设置，位于 `src/cell.rs:50`，缺少此设置 mpv 会为每个实例创建独立窗口
- libmpv 位于 `/opt/homebrew/lib/libmpv.2.dylib`，通过 `build.rs` 链接
- SDL2 位于 `/opt/homebrew/lib/libSDL2-2.0.0.dylib`
- .app 的 rpath 设置为 `@executable_path/../Frameworks/` + `/opt/homebrew/lib` 回退
- 测试视频位于 `test_videos/` 目录（4 个 60s H.264 文件）

## 下一步
- [ ] 用户测试：验证单窗口宫格播放是否正常
- [ ] 文件对话框多选是否生效（macOS 需 Cmd+Click）
- [ ] Windows 交叉编译
- [ ] 拖拽添加视频文件
- [ ] 9/12/16 宫格性能压测
- [ ] DMG 安装包
- [ ] 独立离线 .app（dylibbundler 打包 64 个传递依赖）

## 被阻塞
- 无

## 未解决的问题
- libmpv 有 64 个传递依赖，独立离线打包需 dylibbundler
- Consumer GPU NVDEC 并发限制 2-3 路需文档说明
