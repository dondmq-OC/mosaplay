# Mission Status

> 最后更新: 2026-05-02 GridPlayer v0.1.0 MVP 完成

## 已完成
- [x] 技术调研：mpv vs libVLC vs FFmpeg — libmpv 胜出
- [x] 架构设计：Rust + SDL2 + OpenGL + libmpv
- [x] 多实例 mpv 渲染引擎（FBO + mpv_render_context）
- [x] GPU 硬件加速（Metal on Mac, hwdec=auto-safe）
- [x] Grid 布局计算（3×3, 4×3, 4×4, 2×2, auto）
- [x] 12 个键盘快捷键（播放/暂停/Seek/速度/焦点/全屏/布局切换）
- [x] 异步独立控制每个视频单元
- [x] 极简界面（焦点边框 + 窗口标题状态栏）
- [x] macOS .app 打包（GridPlayer.app, 7.6MB）
- [x] 实测：Apple M4 GPU, OpenGL 4.1 Metal, 4视频零崩溃

## 下一步
- [ ] Windows 交叉编译（需 mpv.dll + SDL2.dll）
- [ ] 拖拽添加视频文件
- [ ] 9/12/16 宫格性能测试
- [ ] DMG 安装包制作
- [ ] 独立离线 .app（bundle 全部 64 个传递依赖）

## 被阻塞
- 无

## 未解决的问题
- libmpv 有 64 个传递依赖，独立离线打包需 dylibbundler
- Windows 版本需在 Windows 环境编译或配置交叉编译
- Consumer GPU 的 NVDEC 并发限制（2-3 路）需在文档中说明
