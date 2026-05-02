# Mission Status

> 最后更新: 2026-05-03 MosaPlay 全项目改名完成

## 项目：MosaPlay — 多宫格视频播放器
- **GitHub**: https://github.com/dondmq-OC/mosaplay
- **macOS .app**: `target/MosaPlay.app` (2.0MB binary)
- **Windows**: GitHub Actions 自动构建 → MosaPlay-win64.zip
- **Icon**: 6 层残影 +90px, 440px 辉光, 暗黑背景

## 已完成功能
- [x] libmpv 多实例引擎 (vo=libmpv, FBO, shared GL context, Metal/D3D11 hwdec)
- [x] 智能布局引擎: 统一宫格 + 分栏 + 宽高比感知
- [x] 鼠标悬停聚焦、中键关闭、Delete 关闭
- [x] 拖放: 格子替换/边距新增/字幕外挂
- [x] 键盘快捷键完整
- [x] macOS .app + Windows portable zip (CI)
- [x] 双语 README + GitHub 完整页面
- [x] 字幕支持 (sub-auto=all, S 键开关, 拖入加载)

## 下一步
- [ ] 独立离线 .app (dylibbundler)
- [ ] DMG 安装包
- [ ] GitHub Release 正式发布
- [ ] 右键菜单 (context menu)
- [ ] 用户反馈收集
