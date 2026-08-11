# OpenCode 等 Coding Agent 项目架构分析与 openrode 实现路线

> 调研基线：2026-08-11。OpenCode 以 `anomalyco/opencode` 的 `dev` 分支当前源码为准；公开文档和 v2 specs 仅用于补充，规划能力不视为已实现能力。

## 1. 结论先行

OpenCode 的核心不是“调用一次 LLM 再执行命令”，而是一个以 **Session 为持久化边界、Message Part 为事件模型、Agent 为策略集合、Tool 为受权限控制的能力、Processor 为流式状态机** 的循环执行系统。

最值得复刻的不是 UI，而是以下六层分离：

1. **模型协议层**：统一不同 provider 的流式文本、推理、工具调用、用量和错误。
2. **会话事件层**：把文本、reasoning、tool call、tool result、step、patch 都保存为可增量更新的 part。
3. **Agent 策略层**：Agent 只描述 prompt、model、采样参数、步数和 permission，不直接承载循环。
4. **工具运行层**：统一 schema 校验、上下文、审批、取消、输出截断和附件。
5. **Agent loop 层**：组装上下文，调用模型，消费流事件，执行工具，判断继续/停止/压缩。
6. **安全与恢复层**：权限判定、外部目录保护、环境文件保护、快照、patch、abort、重试与 compaction。

对 openrode 而言，正确的第一目标不是一次实现完整 OpenCode，而是先做出下面这个“可闭环内核”：

```text
用户输入
  -> Session 写入 UserMessage
  -> 构建 system + history + tools
  -> Provider 流式响应
  -> AssistantMessage / Part 增量落盘
  -> tool_call 参数校验
  -> permission 判定
  -> tool execute
  -> ToolResult 落盘
  -> 再次请求模型
  -> 无 tool_call 或达到终止条件
  -> 最终回复
```

## 2. OpenCode 仓库框架

OpenCode 是一个 TypeScript/Bun monorepo。顶层 `packages/` 将产品壳、协议、SDK 和核心运行时拆开；真正的 agent 业务集中在 `packages/opencode/src/`。

### 2.1 顶层包的职责

| 包 | 主要职责 | 对 openrode 的启示 |
|---|---|---|
| `packages/opencode` | Agent、session、tool、provider、permission、MCP、LSP、server 等核心业务 | 对应未来 `openrode-core` |
| `packages/core` | 公共 schema、Effect service、数据库、模型/会话基础类型 | 稳定领域类型应从 CLI 中独立 |
| `packages/llm` | 模型流与跨 provider 事件抽象 | `hillm` 应演进成 provider adapter，而不是只包 HTTP |
| `packages/tui` / `app` / `desktop` | 多种交互前端 | 内核通过事件/API 服务多个前端 |
| `packages/sdk` / `sdk-next` / `client` | 外部调用接口与生成客户端 | 核心 API 稳定后再做 |
| `packages/plugin` | 插件 API | 工具、hooks、认证适配不应硬编码进循环 |
| `packages/protocol` / `schema` | 边界协议与 schema | 持久化 schema 与 provider schema 分离 |

### 2.2 `packages/opencode/src` 核心域

| 目录 | 职责 |
|---|---|
| `agent/` | Agent 定义、内置 Agent、配置合并、子 Agent 权限派生 |
| `session/` | 会话、消息、prompt 解析、循环、流处理、压缩、摘要、回滚、状态 |
| `tool/` | 工具协议、注册表、内置工具、输出截断、task 子 Agent 工具 |
| `permission/` | ordered ruleset、ask/allow/deny、审批请求生命周期 |
| `provider/` | provider/model 发现、认证、模型参数变换、消息兼容 |
| `mcp/` | MCP client、工具和资源接入 |
| `plugin/` | hooks 和自定义工具扩展 |
| `snapshot/` / `patch/` | 工具执行前后工作区状态与变更记录 |
| `config/` | 全局、项目、Markdown agent/command 等配置装载与合并 |
| `server/` / `bus/` | API 与事件分发，解耦核心和 UI |
| `background/` | 后台子任务生命周期 |

## 3. OpenCode Agent 架构

### 3.1 Agent 本身是配置，不是执行器

`agent/agent.ts` 的 `Agent.Info` 最重要字段如下：

```text
name, description
mode: primary | subagent | all
hidden, native
model: providerID + modelID
prompt
temperature, topP, variant, options
steps
permission: ordered ruleset
```

`Agent.Service` 提供 `get/list/defaultInfo/defaultAgent/generate`。它读取全局配置和技能目录，构造内置 agents，再按用户配置覆盖。执行循环在 `SessionPrompt`，事件消费在 `SessionProcessor`。这种设计使同一个 loop 可以运行 build、plan、explore 或自定义 agent。

当前内置角色：

| Agent | 类型 | 作用 | 默认权限特征 |
|---|---|---|---|
| `build` | primary | 默认执行型 agent | 工具大体允许；问题和进入计划模式允许 |
| `plan` | primary | 只读规划 | 普通 edit 禁止，仅计划文件例外；可退出 plan |
| `general` | subagent | 通用多步子任务 | 基于默认规则，禁 todo write |
| `explore` | subagent | 快速代码库搜索 | deny-all 后仅放行 read/grep/glob/list/bash/web 等探索能力 |
| `compaction` | hidden primary | 上下文压缩 | 禁用工具 |
| `title` | hidden primary | 生成会话标题 | 禁用工具 |
| `summary` | hidden primary | 摘要 | 禁用工具 |

配置合并顺序本质上是：

```text
安全默认值 -> 内置 Agent 专属规则 -> 用户全局规则 -> 用户 Agent 覆盖
```

需要特别保留“有序规则、最后匹配者生效”的语义，否则 `* deny` 后对特定命令 `allow` 的表达能力会丢失。

### 3.2 Session 是运行与持久化边界

Session 记录标题、父子关系、当前 agent/model、权限覆盖、时间和状态。Message 分 user/assistant；Message 下再拆 Part，而不是把一次回复保存成单个字符串。

关键 Part 最小集合：

| Part | 含义 |
|---|---|
| `text` | 用户或助手文本，可流式 append |
| `reasoning` | 推理流，可有 provider metadata |
| `tool` | 一次工具调用，内部具有 pending/running/completed/error 状态 |
| `file` | 输入附件或工具输出附件 |
| `step-start` / `step-finish` | 一次模型 step 边界、finish reason、tokens、cost |
| `snapshot` / `patch` | 工具执行前后的代码库状态和变更 |
| `compaction` | 上下文压缩边界与摘要 |

这种事件化表示支持：流式 UI、断线恢复、工具审批后续跑、取消、成本统计、会话回放、父子任务导航和 diff 展示。

### 3.3 一次 prompt 的真实路径

```text
SessionPrompt.prompt(input)
  1. 读取 session，清理 revert 状态
  2. 将用户输入解析成 text/file/agent/resource parts
  3. 持久化 UserMessage 与 parts
  4. 合并本轮 tools 覆盖为 session permission
  5. 进入 loop(sessionID)

loop
  1. 读取历史，找到最后用户消息及 agent/model
  2. 检查 pending compaction / continuation
  3. 生成 AssistantMessage
  4. 解析 system prompt、项目 instructions、skills、附件
  5. 从 registry + MCP + plugin 收集工具
  6. 用 agent + session rules 过滤/包装工具
  7. 调用 LLM.stream
  8. SessionProcessor 消费标准化事件
  9. 若有 tool call，执行并写回结果，继续下一 step
 10. 若正常结束、被阻塞、达到 steps 或需压缩，则退出或转 compaction
```

重要点：循环消费的是归一化 `StreamEvent`，而不是直接依赖 OpenAI SSE 字段。这样 provider 差异被限制在模型适配层。

### 3.4 SessionProcessor 是流式状态机

Processor 在模型流开始前创建快照，维护：

- 当前 assistant message；
- tool call ID 到 part 的映射；
- reasoning ID 到 reasoning part 的映射；
- 当前 text part；
- `blocked`、`shouldBreak`、`needsCompaction`；
- token、cost、finish reason 和 snapshot。

它对事件做确定性转换：

| 输入事件 | 状态变更 |
|---|---|
| `text-start/delta/end` | 创建、增量更新、完成 text part |
| `reasoning-start/delta/end` | 创建、增量更新、完成 reasoning part |
| `tool-input-start/delta/end` | 构造工具参数 part |
| `tool-call` | schema 校验，pending -> running |
| `tool-result` | running -> completed，标准化 title/output/metadata/attachments |
| `tool-error` | running -> error |
| `step-start` | 保存起始 snapshot |
| `step-finish` | 保存 usage/cost/finish reason，计算 patch |
| provider error | 归一化错误并结束或按策略重试 |

工具结果不是任意 JSON 直接塞回 UI，而是规范化为 `title + metadata + output + attachments`；超长输出由 truncate 层处理。这一点会显著降低上层复杂度。

### 3.5 Tool 架构

OpenCode Tool 接口可抽象为：

```text
Tool = {
  id
  description(agent)             // 可按 agent 动态生成
  parameters                     // JSON Schema / typed schema
  execute(args, context) -> {
    title, metadata, output, attachments?
  }
}

ToolContext = {
  sessionID, messageID, agent, callID
  abort
  messages
  metadata(update)
  ask(permission request)
}
```

执行前后还有插件 hooks，例如 tool execute before/after。内置工具、插件工具、MCP 工具最终都进入同一注册和调用路径。

### 3.6 权限不是布尔工具开关

权限判定输入至少为：

```text
permission key + resource patterns + ordered ruleset + session + tool metadata
```

输出为 `allow | ask | deny`。常见 key 包括 read、edit、bash、task、external_directory、skill、web、LSP、question 和 doom_loop。

应实现的安全细节：

- workspace 外路径单独经过 `external_directory`；
- `.env` 和 `.env.*` 默认 ask，example 可读；
- bash 根据结构化命令/argv 或明确 pattern 判定，不能只做字符串前缀；
- deny 的工具/子 Agent 尽量不暴露给模型，减少无效调用；
- ask 是可恢复的暂停状态，而非抛出普通错误；
- Agent 规则与 session 临时规则合并；
- 重复相同工具调用触发 doom-loop 防护；
- 子 Agent 权限必须显式派生，不能意外越权或无限递归。

### 3.7 子 Agent：Task 是一个普通工具

Task tool 的实现方式很关键：

1. 根据当前 agent 的 `permission.task` 检查目标 subagent；
2. 获取目标 Agent 配置；
3. 从 parent session permission 与 subagent permission 派生 child rules；
4. 默认禁止 child 再调用 task（除非明确配置），并限制 primary-only tools；
5. 创建带 `parentID` 的 child session；
6. 使用目标 agent/model 在 child session 调用普通 `prompt`；
7. 前台模式等待最终结果；后台模式登记 job 并立即返回；
8. 将 child session ID、结果或错误写回父 session 的 tool part。

所以子 Agent 不是第二套运行时，而是“创建子 Session + 复用同一 prompt/loop”的递归组合。这使审计、取消、权限和 UI 导航天然一致。

风险点：规则继承若定义不清，会出现子 Agent 阻塞、越权或递归爆炸。建议 openrode 早期硬编码 `max_agent_depth = 1`，成熟后才开放显式递归策略。

### 3.8 上下文工程与 compaction

上下文并非简单发送全部历史。进入模型前需要：

- system prompt：产品、provider、agent prompt；
- 项目 instructions：如 AGENTS.md；
- message history：过滤无效/中断 part，转换 provider 格式；
- 文件、图片和 MCP resource；
- tool schema；
- 权限后真正可用的工具列表；
- 接近 context window 时的 compaction summary；
- 最大步数提示与 continuation 提示。

Compaction 应是独立隐藏 agent，不拥有工具。摘要必须保留目标、已完成工作、关键决策、修改文件、未完成项、错误和下一步，而不是普通聊天摘要。

## 4. 与其他项目的对照

| 项目 | 强项 | 建议吸收 | 不宜照搬 |
|---|---|---|---|
| OpenCode | Session/Part 事件模型、可配置 Agent、工具/MCP/插件统一、父子 Session | 作为总体架构参考 | 当前 TS/Effect 复杂度与快速演进中的双版本抽象 |
| OpenAI Codex | Rust core、UI 与业务分离、OS 级 sandbox、结构化 exec policy | Rust crate 边界、沙箱和审批必须进入基础架构 | 一开始实现所有 OS sandbox 后端 |
| Aider | repo map、tree-sitter 符号图、受约束 edit format、git-first | 大仓库上下文选择和可验证编辑 | 把全部能力绑定在特定编辑格式/聊天 coder 类层级 |
| Goose | Rust、多 provider、MCP 优先、CLI/Desktop/API 共核 | provider trait、extension/MCP 边界 | MVP 同时追求通用自动化和 coding agent 两种产品定位 |
| Claude Code | Agents、skills、hooks、插件、成熟的权限/产品交互 | hooks 生命周期、渐进加载、子 Agent 可观测性 | 闭源内核无法作为源码级实现依据 |

综合取舍：以 OpenCode 的 session/agent/tool 模型为骨架，以 Codex 的 Rust core + sandbox 为安全底座，以 Aider 的 repo map 作为后续上下文优化，以 Goose 的 MCP 作为扩展标准。

## 5. 当前 openrode 差距

当前仓库已有：

- Rust workspace；
- `hillm` 中 OpenAI-compatible chat 和 chat stream 客户端；
- messages/request/response 的部分 serde 类型；
- CLI 接收单个 prompt；
- `src/tool.rs` 中 Read/Write/Bash 的雏形。

当前距离 agent 闭环仍缺：

1. SSE parser 按 HTTP chunk 直接 `lines()`，无法处理 JSON 跨 chunk，必须先修；
2. assistant tool call delta 尚未组装成完整调用；
3. 没有 provider-neutral stream event；
4. 没有 Session、Message/Part ID、存储和恢复；
5. 没有 tool trait、registry、schema 校验和 executor；
6. 没有 permission/approval/sandbox；
7. 没有循环终止、最大步数、取消和重试；
8. 没有快照/diff、compaction、项目指令和 context selection；
9. `src/tool.rs` 当前缺少可见 imports/type 定义，且依赖未在模块树中接通；
10. CLI、agent core 与 provider client 仍耦合在一次调用流程中。

## 6. 建议的 Rust 框架

```text
openrode/
├── crates/
│   ├── hillm/                 # provider adapters + normalized LLM stream
│   ├── openrode-core/         # domain types, agent loop, session services
│   ├── openrode-tools/        # read/write/patch/bash/search/list
│   ├── openrode-policy/       # permission rules, approval, command policy
│   ├── openrode-storage/      # SQLite repositories and migrations
│   ├── openrode-sandbox/      # process/filesystem isolation abstraction
│   ├── openrode-mcp/          # MCP client and tool adapter（后期）
│   └── openrode-cli/          # clap/TUI adapter
└── src/main.rs                # 过渡期 binary，最终可移入 openrode-cli
```

MVP 可以先减少 crate 数量，但模块边界应保留：

```rust
pub trait ModelProvider {
    fn stream(&self, req: ModelRequest)
        -> Pin<Box<dyn Stream<Item = Result<ModelEvent, ModelError>> + Send>>;
}

pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    async fn execute(&self, args: Value, ctx: ToolContext)
        -> Result<ToolOutput, ToolError>;
}

pub trait SessionStore {
    async fn append_part(&self, part: Part) -> Result<()>;
    async fn update_part(&self, part: Part) -> Result<()>;
    async fn history(&self, session: SessionId) -> Result<Vec<MessageWithParts>>;
}

pub trait PermissionEngine {
    async fn check(&self, req: PermissionRequest) -> PermissionDecision;
}
```

### 6.1 建议领域类型

```text
AgentSpec
  id, description, mode, model, prompt, sampling, max_steps, permissions

Session
  id, parent_id, title, agent_id, model_id, state, permission_overrides

Message
  id, session_id, role, created_at, model, agent

Part
  Text | Reasoning | Tool | File | StepStart | StepFinish | Patch | Compaction

ToolState
  Pending(input_raw) | Running(input, started_at)
  | Completed(input, output, ended_at) | Error(input?, error, ended_at)

ModelEvent
  TextStart/Delta/End
  | ReasoningStart/Delta/End
  | ToolInputStart/Delta/End
  | ToolCall | ToolResult | Usage | Finish | Error
```

所有 ID 应可排序（ULID/UUIDv7），以简化流式 part 排序与分页。

## 7. 最小功能单元（按依赖顺序）

每个单元应可独立测试和合并。

### Phase 0：稳定模型流

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P0-01 | 增量 SSE decoder | 缓冲残片，以空行分隔 event，支持多 `data:` 与 `[DONE]` | JSON 跨 2~3 个网络 chunk 仍正确解析 |
| P0-02 | Tool call delta assembler | 按 choice/index/id 合并 name 与 arguments delta | 分片参数最终形成合法 JSON |
| P0-03 | 归一化 `ModelEvent` | 将 provider response 映射为 text/reasoning/tool/usage/finish | loop 不引用 OpenAI response 类型 |
| P0-04 | Provider trait | OpenAI-compatible 实现放到 trait 后 | fake provider 可完全驱动测试 |
| P0-05 | 错误分类 | auth/rate-limit/context/network/invalid-response/cancelled | 每类决定 retryable 与用户消息 |

### Phase 1：单 Agent 工具闭环

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P1-01 | `ToolSpec` | id、description、JSON Schema | 可序列化给模型 |
| P1-02 | `Tool` trait | typed/JSON args + context + output | fake tool 可记录调用 |
| P1-03 | Registry | register/get/list、重复 ID 报错 | 确定性顺序 |
| P1-04 | 参数校验 | 执行前 JSON Schema validate | 错误作为 tool result 反馈模型 |
| P1-05 | Read tool | workspace 路径解析、范围读取、二进制拒绝 | 禁止目录穿越 |
| P1-06 | Write/Patch tool | 原子写入，优先 patch | 失败不留下半文件 |
| P1-07 | Bash tool | argv/cwd/timeout/取消/输出上限 | 超时杀死进程树 |
| P1-08 | Agent loop | 模型 -> tool -> 模型，支持 max steps | fake 两步场景得到最终文本 |
| P1-09 | Abort | 一个 cancellation token 贯穿模型和工具 | Ctrl-C 后进程和流均停止 |
| P1-10 | Doom loop | 规范化 tool+args 哈希，连续重复阈值 | 重复调用转审批/终止 |

### Phase 2：Session 与可恢复状态

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P2-01 | ID 与时间 | UUIDv7/ULID newtypes | 排序稳定且避免混用 |
| P2-02 | Message/Part schema | tagged enum + schema version | round-trip 测试 |
| P2-03 | SQLite migration | session/message/part 表 | 新库自动迁移 |
| P2-04 | append/update part | 事务、状态转换校验 | running 不能倒退为 pending |
| P2-05 | 流式持久化 | delta 合批写入，finish 强制 flush | 中断后能看到已有输出 |
| P2-06 | resume/list | 列会话、加载历史、继续 prompt | 进程重启后可继续 |
| P2-07 | Event bus | `SessionEvent` broadcast | CLI 与存储消费同一事件 |
| P2-08 | usage/cost | step usage 聚合 | session 总数与 step 和一致 |

### Phase 3：Agent 与权限

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P3-01 | `AgentSpec` | primary/subagent/all、prompt/model/steps | 配置 round-trip |
| P3-02 | 内置 build/plan | 两套 permission defaults | plan 无法写普通源文件 |
| P3-03 | ordered matcher | permission+pattern，last match wins | 表驱动覆盖冲突规则 |
| P3-04 | path policy | canonicalize、workspace/external、symlink | symlink 不可逃逸 |
| P3-05 | command policy | 解析 shell 命令段，按 argv/pattern 判定 | `safe && unsafe` 不能整体误放行 |
| P3-06 | approval broker | request/pending/approve-once/always/reject | ask 后 loop 可恢复 |
| P3-07 | secret file policy | `.env*` 等敏感规则 | 默认 ask |
| P3-08 | tool visibility | deny 的工具不发给模型 | schema 列表匹配权限结果 |
| P3-09 | config layering | defaults/global/project/agent/session | 冲突结果可解释 |

### Phase 4：工程上下文与恢复

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P4-01 | Instructions loader | 从 cwd 向上/向下发现 AGENTS.md | 合并顺序稳定 |
| P4-02 | `.gitignore` filter | 搜索/读取默认尊重 ignore | fixture 测试 |
| P4-03 | output truncation | head/tail + 落临时文件 | token/byte 上限稳定 |
| P4-04 | pre/post snapshot | git diff 或文件 hash | 写工具后生成 changed files |
| P4-05 | retry policy | 指数退避+jitter+Retry-After | fake clock 测试 |
| P4-06 | context budget | 按模型窗口预算 system/history/tools | 永不超预算提交 |
| P4-07 | compaction | 隐藏 summary agent + compaction part | 压缩后目标与未完成项仍在 |
| P4-08 | revert | 基于 snapshot/patch 恢复 agent 修改 | 用户原有脏修改不被覆盖 |

### Phase 5：子 Agent

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P5-01 | Child session | `parent_id` 与导航查询 | parent 可列 child |
| P5-02 | Task tool | 目标 agent + prompt -> child loop | child 结果回到父 tool result |
| P5-03 | 权限派生 | parent session 与 child agent 明确定义合并 | child 永不比策略允许的更强 |
| P5-04 | 深度限制 | 默认 depth=1、显式 max depth | 递归在创建前拒绝 |
| P5-05 | 前台取消传播 | parent abort -> child abort | 无孤儿进程/请求 |
| P5-06 | 后台 job | queued/running/completed/error/cancelled | 父 loop 无需轮询收到通知 |
| P5-07 | 结果压缩 | 超长 child 输出摘要并保留 session 引用 | 父上下文不被撑爆 |
| P5-08 | 并发限流 | 全局+每 session semaphore | 压测不超配置并发 |

### Phase 6：扩展与产品层

| ID | 最小单元 | 实现 | 验收 |
|---|---|---|---|
| P6-01 | Hook bus | session/message/tool/permission 生命周期 | hook 错误策略明确 |
| P6-02 | MCP client | initialize/list/call/resources/cancel | 官方测试 server 互通 |
| P6-03 | MCP tool adapter | schema/name/结果转 `Tool` | 与内置工具走相同权限 |
| P6-04 | Skills | 发现元数据、按需读取、权限控制 | 未加载正文不进上下文 |
| P6-05 | Repo map | tree-sitter symbols + ranking | 大仓库在 token 预算内命中相关符号 |
| P6-06 | Server API | prompt/abort/events/approval/session | CLI 可只做 API client |
| P6-07 | Observability | tracing、tool latency、tokens、cost | 一个 session 可端到端关联 |

## 8. 推荐实施里程碑

### M1：可测试的工具循环

范围：P0 + P1。先不做 SQLite、MCP、子 Agent和 TUI。用内存 history 和 fake provider 验证 agent loop；真实 provider 只做兼容性测试。

完成定义：模型能够调用 read/patch/bash；工具结果返回模型；最终文本输出；有 max steps、timeout、abort、输出截断和路径保护。

### M2：持久化与审批安全

范围：P2 + P3。引入 SQLite、事件总线、build/plan、ordered permission 和可恢复 approval。

完成定义：进程退出后可恢复；每个工具动作可审计；ask/deny/allow 行为确定；workspace 外、symlink 和敏感文件有测试。

### M3：面向真实代码库

范围：P4。加入 instructions、ignore、snapshot/diff、context budget 和 compaction。

完成定义：可在中型仓库连续执行多轮，不因输出或历史无限增长失败；能展示并安全回滚 agent 变更。

### M4：多 Agent 与扩展

范围：P5 + P6 的 MCP/skills/hooks。

完成定义：Task 创建可观察、可取消、有限深度的 child session；内置/MCP/plugin 工具共享 schema、权限和事件路径。

## 9. 测试策略

Agent 系统不能主要依赖在线模型做单测。建议测试金字塔：

1. **纯函数表驱动测试**：permission、path、command、config merge、状态转换；
2. **scripted provider 测试**：预置 `ModelEvent` 序列，覆盖工具循环、错误、重试、compaction；
3. **fake tool 测试**：成功、超时、取消、超长输出、非法 schema；
4. **SQLite crash tests**：在 delta/tool running 时模拟退出再恢复；
5. **filesystem fixtures**：symlink、dirty git、ignored files、二进制和大文件；
6. **provider contract tests**：少量在线/录制 fixture，验证 OpenAI-compatible 差异；
7. **端到端 golden transcript**：输入固定任务，断言 part/event 序列而非自然语言逐字相等。

最关键不变量：

- 每个 tool call 最终恰有一个 completed/error/cancelled 终态；
- assistant delta 顺序可重放；
- deny 永不执行，ask 未批准永不执行；
- session 中断后不留下运行中的子进程；
- child 权限不发生隐式提升；
- compaction 前后任务约束保持；
- agent 写入不覆盖用户已有未提交修改。

## 10. 关键设计决策

1. **先事件模型，后 UI**：否则 TUI 会反向绑死核心状态。
2. **Provider-neutral events 是硬边界**：不要让 OpenAI `choices[].delta` 泄漏到 loop。
3. **权限与沙箱是两层**：权限决定“应不应该”，OS sandbox 保证“即使错了也做不到”。
4. **SessionStore 是事实源**：内存只做缓存，事件先有一致语义再推 UI。
5. **工具输出必须受控**：统一截断、附件化、metadata，不能任由 stdout 注入上下文。
6. **子 Agent 复用同一 loop**：不要创建第二套执行逻辑。
7. **后台任务晚于前台 Task**：先证明 child session、取消和权限，再加并发。
8. **编辑优先 patch + snapshot**：直接 write whole-file 风险高且不利于审计。
9. **配置必须可解释**：调试输出应能显示某次 allow/ask/deny 命中了哪条规则。
10. **v1 schema 预留版本**：Agent、Session、Part、permission 从第一天带 schema version/migration。

## 11. 来源

- [OpenCode 仓库](https://github.com/anomalyco/opencode)
- [OpenCode 核心源码目录](https://github.com/anomalyco/opencode/tree/dev/packages/opencode/src)
- [Agent 实现](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/agent/agent.ts)
- [Session prompt / loop](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/session/prompt.ts)
- [流式 SessionProcessor](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/session/processor.ts)
- [Task 子 Agent 工具](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/tool/task.ts)
- [Tool 协议](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/tool/tool.ts)
- [OpenCode Agents 文档](https://opencode.ai/docs/agents/)
- [OpenCode Permissions 文档](https://opencode.ai/docs/permissions/)
- [OpenCode Skills 文档](https://opencode.ai/docs/skills)
- [OpenAI Codex Rust workspace](https://github.com/openai/codex/blob/main/codex-rs/README.md)
- [Codex core 与 OS sandbox](https://github.com/openai/codex/blob/main/codex-rs/core/README.md)
- [Codex exec policy](https://github.com/openai/codex/blob/main/codex-rs/execpolicy/README.md)
- [Aider 仓库](https://github.com/Aider-AI/aider)
- [Goose 仓库](https://github.com/aaif-goose/goose)
- [Claude Code 仓库与插件示例](https://github.com/anthropics/claude-code)

