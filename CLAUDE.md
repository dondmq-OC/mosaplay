# CLAUDE.md — Long-Running Task Agent

## 编码原则 — Andrej Karpathy 四项准则

### P1: Think Before Coding（先思考，再编码）
- **明确陈述假设**，不确定就问，不要猜
- 存在歧义时**展示多种解读**，不要静默选择
- **该反驳就反驳**——有更简单方案时先说出来
- **困惑时停下来**，命名不清晰之处，请求澄清
- **展示权衡**——多种可行方案时，呈现利弊

### P2: Simplicity First（简单优先）
- 只实现**明确要求**的功能，不预先设计
- 不为单一用途创建抽象层
- 不做未被要求的"灵活性"或"可配置性"
- 不对不可能发生的场景做错误处理
- 如果 200 行能写成 50 行，重写成 50 行

### P3: Surgical Changes（外科手术式改动）
- **只改你该改的，清理只清理你弄乱的**
- 不"顺手改进"相邻代码、注释、格式化
- 不重构没坏的东西
- 匹配现有风格，哪怕你觉得不同
- 发现无关死代码→提出来，但不删除
- 你的改动产生的孤儿→清理掉（import、变量、函数）

### P4: Goal-Driven Execution（目标驱动执行）
- 定义成功标准，循环直到验证通过
- "添加验证" → "为无效输入写测试，让测试通过"
- "修复 bug" → "写复现测试，让测试通过"
- "重构 X" → "确保测试前后均通过"
- 多步骤任务给简短计划：`[步骤] → 验证: [检查]`

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

### caffeinate 包装
- 预计超过 30 秒的 Bash 命令必须用 `caffeinate -dims` 包装：
  ```bash
  caffeinate -dims <your-long-command>
  ```
- `-dims` = 防止显示器、系统空闲、硬盘、系统全部休眠
- 编译、安装依赖、网络下载等操作一律包装

### 定期 Checkpoint（隐式 Keep-Alive）
- Cron 任务每 30 分钟自动触发一次轻量 checkpoint
- 验证 `tasks/mission.md` 与当前进度一致
- 保持 session 活跃，防止空闲超时

### 跨会话存活
- 即使系统休眠或合盖，checkpoint 系统保证状态不丢
- 下次会话启动时，从 `tasks/mission.md` 无缝恢复

## 6. Sandbox 准则

- Agent 通过标准的 execute 方法与 Sandbox 交互
- 凭证和敏感信息不进入 Agent 上下文
- Sandbox 提供安全隔离，Agent 对凭证无感
- 需要提权的操作使用 `dangerouslyDisableSandbox: true`（需用户批准）

## 7. 开发方法通则

- 不能决策是因为缺少评分函数，不是缺少讨论
- 第二次手动操作时就应该被编码
- 先查约束，再查代码（约束不可改，代码可改）
- 用户路径必然与开发者不同；在用户环境运行最小原型，不要增测试用例
- 当 if-else 链出现第五个特例时——停止写规则，开始枚举候选
- 并排对比 > 逐个判断（序贯决策被近因和锚定效应扭曲）
- 验证链的强度取决于层间间隙，不是单层强度；最危险的假设最先证伪（成本最低收益最高）
