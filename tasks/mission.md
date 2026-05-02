# Mission Status

> 最后更新: 2026-05-02 Windows 构建成功

## 项目：GridPlayer — 多宫格视频播放器
- **GitHub**: https://github.com/dondmq-OC/gridplayer (22 commits)
- **macOS .app**: `target/GridPlayer.app`
- **Windows**: GitHub Actions 自动构建 → GridPlayer-win64.zip (便携免安装)
- **Loop**: 95c2789d (:23,:53)

## 已完成功能
- [x] libmpv 多实例引擎 (vo=libmpv, FBO, shared GL context, hwdec)
- [x] 智能布局引擎 v3: 统一宫格 + 分栏(竖屏独立列) + 列宽动态搜索
- [x] 鼠标悬停自动切换焦点、中键关闭、Delete 键关闭
- [x] 拖放: 拖到格子替换, 拖到边距新增
- [x] 键盘: Space/←→/↑↓/[]/M/Tab/1-9/F/G/Esc/R/Delete
- [x] macOS .app + 重影播放按钮图标
- [x] 文件对话框：中文, 多选
- [x] Karpathy 四项原则写入 CLAUDE.md
- [x] 测试视频: 13 个 (多种 AR/编码/分辨率/码率)
- [x] 崩溃修复: mpv params 哨兵, 渲染器 glViewport 重写

## 下一步
- [ ] 独立离线 .app (dylibbundler 打包 64 传递依赖)
- [ ] DMG 安装包
- [ ] GitHub Release 发布双平台版本
