# CLAUDE.md — Long-Running Task Agent

## 0. Git Workflow
- **任务开始前**: 运行 `git log --oneline -10` 查看最近历史，理解上下文。
- **任务完成后**: 提交变更：
  ```bash
  git add -A
  git commit -m "feat/fix/refactor: <简短描述>"
  ```
- 提交信息遵循 conventional commits 格式。
- 不提交包含 secrets 的文件（.env, credentials.* 等）。

## 1. 上下文管理 — Checkpoint System

### 触发条件
当对话上下文接近 70% 窗口容量，或收到压缩信号时，**必须**执行 checkpoint。

### Checkpoint 流程
1. 将当前状态写入 `tasks/mission.md`，包含：
   - **已完成** — 每个条目附带修改过的文件路径列表
   - **下一步** — 按优先级排列
   - **被阻塞** — 阻塞原因
   - **未解决的问题** — 需要决策或信息
2. 将完整修改文件清单写入 `tasks/modified_files.md`
3. 执行 checkpoint 提交：
   ```bash
   git add tasks/ CLAUDE.md .claude/session_log/
   git commit -m "checkpoint: <当前阶段摘要>"
   ```

### 交接模式（上下文重置后恢复）
1. 首先读取 `tasks/mission.md` 了解当前状态
2. 读取 `.claude/session_log/events.jsonl` 尾部 50 条事件恢复上下文
3. 优先处理无阻塞的任务，不卡在已阻塞项上

### 上下文压缩前检查清单
- [] `tasks/mission.md` 已更新
- [] `tasks/modified_files.md` 包含所有变更文件
- [] 关键决策信息已写入 mission（不依赖记忆）
- [] 已提交 checkpoint

## 2. 错误处理与重试

- 任何操作最多**重试 3 次**
- 3 次后仍无进展 → 写入 `tasks/pending_for_human.md`，包含：
  - 尝试了什么
  - 失败原因
  - 建议的解决方向
- **不要卡住**：记录后立即转到下一个任务

## 3. 崩溃恢复

当会话重新启动时：
1. 检查 `tasks/mission.md` — 如果存在且有未完成项，从上个 checkpoint 恢复
2. 读取 `.claude/session_log/events.jsonl` 获取完整上下文
3. 检查 `.claude/session_log/checkpoints/` 中最新的快照
4. 验证 `git status` 确认工作区状态

## 4. Session 持久化日志

日志位于 `.claude/session_log/events.jsonl`，追加写入，每行一个 JSON 事件：
```json
{"ts":"<ISO8601>","type":"<event_type>","summary":"<描述>","files":["<path>"],"decision":"<关键决策>"}
```

事件类型：`task_start`, `task_complete`, `checkpoint`, `error`, `blocked`, `decision`, `file_change`

**读取事件**：从任意位置恢复时，读取尾部 N 行即可回溯上下文。

## 5. 防休眠

- 长时间运行的 Bash 命令通过 `caffeinate` 包装
- 定期 checkpoint 写盘作为隐式 keep-alive
- 关键操作前确保系统不休眠

## 6. Sandbox 准则

- Agent 通过标准的 execute 方法与 Sandbox 交互
- 凭证和敏感信息不进入 Agent 上下文
- Sandbox 提供安全隔离，Agent 对凭证无感
- 需要提权的操作使用 `dangerouslyDisableSandbox: true`（需用户批准）
