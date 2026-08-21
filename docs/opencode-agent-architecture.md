<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>拆解 OpenCode</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500;12..96,700;12..96,800&family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap">
<style>
:root{
  --ground:#f4f2ec;
  --surface:#fffdf8;
  --surface-2:#ece9e0;
  --ink:#20242a;
  --ink-soft:#4c5560;
  --muted:#6e7883;
  --line:#d8d3c6;
  --accent:#a85f0a;
  --accent-soft:#f3e3cd;
  --teal:#1f7465;
  --teal-soft:#dcebe6;
  --code-bg:#14181f;
  --code-ink:#e6e1d6;
  --code-line:#2a3140;
  --code-muted:#8b95a5;
  --code-amber:#e8a94e;
  --code-teal:#7cc4b4;
  --err:#b0413c;
  --ok:#3d7a41;
  --chip:#e7e2d5;
}
@media (prefers-color-scheme: dark){
  :root:not([data-theme="light"]){
    --ground:#0e1116;
    --surface:#151a21;
    --surface-2:#1b212b;
    --ink:#e7e2d6;
    --ink-soft:#b9c0ca;
    --muted:#8b95a5;
    --line:#2b3240;
    --accent:#e8a94e;
    --accent-soft:#2c2415;
    --teal:#6fc0ae;
    --teal-soft:#152925;
    --code-bg:#10141a;
    --code-ink:#e6e1d6;
    --code-line:#242c39;
    --code-muted:#7d8798;
    --code-amber:#e8a94e;
    --code-teal:#7cc4b4;
    --err:#e5726e;
    --ok:#7bc47f;
    --chip:#232a35;
  }
}
:root[data-theme="dark"]{
  --ground:#0e1116;
  --surface:#151a21;
  --surface-2:#1b212b;
  --ink:#e7e2d6;
  --ink-soft:#b9c0ca;
  --muted:#8b95a5;
  --line:#2b3240;
  --accent:#e8a94e;
  --accent-soft:#2c2415;
  --teal:#6fc0ae;
  --teal-soft:#152925;
  --code-bg:#10141a;
  --code-ink:#e6e1d6;
  --code-line:#242c39;
  --code-muted:#7d8798;
  --code-amber:#e8a94e;
  --code-teal:#7cc4b4;
  --err:#e5726e;
  --ok:#7bc47f;
  --chip:#232a35;
}
*{box-sizing:border-box;margin:0;padding:0}
html{scroll-behavior:smooth}
body{
  background:var(--ground);
  color:var(--ink);
  font-family:"IBM Plex Sans","PingFang SC","Hiragino Sans GB","Microsoft YaHei",sans-serif;
  font-size:16px;line-height:1.75;
  -webkit-font-smoothing:antialiased;
}
.mono{font-family:"IBM Plex Mono",ui-monospace,"SF Mono",Menlo,monospace}
.wrap{max-width:900px;margin:0 auto;padding:0 24px}

/* ---------- header ---------- */
header{padding:72px 0 40px;border-bottom:1px solid var(--line)}
.eyebrow{
  font-family:"IBM Plex Mono",monospace;font-size:12px;letter-spacing:.18em;
  text-transform:uppercase;color:var(--accent);margin-bottom:18px
}
h1{
  font-family:"Bricolage Grotesque","PingFang SC","Microsoft YaHei",sans-serif;
  font-weight:800;font-size:clamp(34px,6vw,56px);line-height:1.12;
  letter-spacing:-.01em;text-wrap:balance;margin-bottom:20px
}
.lede{font-size:18px;color:var(--ink-soft);max-width:46em}
.lede strong{color:var(--ink)}
.meta-row{display:flex;flex-wrap:wrap;gap:10px;margin-top:26px}
.meta-pill{
  font-family:"IBM Plex Mono",monospace;font-size:12px;color:var(--ink-soft);
  background:var(--chip);border:1px solid var(--line);border-radius:4px;
  padding:4px 10px
}

/* ---------- nav ---------- */
nav{
  position:sticky;top:0;z-index:20;background:var(--ground);
  border-bottom:1px solid var(--line);
}
nav .wrap{display:flex;gap:4px;overflow-x:auto;padding-top:10px;padding-bottom:10px}
nav a{
  flex:0 0 auto;font-family:"IBM Plex Mono",monospace;font-size:12.5px;
  color:var(--muted);text-decoration:none;padding:5px 11px;border-radius:5px;
  white-space:nowrap
}
nav a:hover{color:var(--ink);background:var(--chip)}
nav a:focus-visible{outline:2px solid var(--accent);outline-offset:1px}

/* ---------- sections ---------- */
section{padding:56px 0 8px}
.sec-head{display:flex;align-items:baseline;gap:14px;margin-bottom:22px}
.sec-num{
  font-family:"IBM Plex Mono",monospace;font-size:13px;color:var(--accent);
  border:1px solid var(--accent);border-radius:4px;padding:1px 8px;flex:0 0 auto
}
h2{
  font-family:"Bricolage Grotesque","PingFang SC","Microsoft YaHei",sans-serif;
  font-weight:700;font-size:clamp(24px,3.4vw,32px);line-height:1.25;text-wrap:balance
}
h3{
  font-weight:600;font-size:19px;margin:36px 0 12px;
  padding-top:10px
}
h3 .tag{
  font-family:"IBM Plex Mono",monospace;font-size:11px;color:var(--teal);
  background:var(--teal-soft);border-radius:4px;padding:2px 7px;
  vertical-align:2px;margin-right:8px;letter-spacing:.05em
}
p{margin:0 0 14px;max-width:56em}
p.tight{margin-bottom:8px}
ul,ol{margin:0 0 14px;padding-left:1.4em;max-width:56em}
li{margin-bottom:6px}
strong{font-weight:600}
a.inline{color:var(--accent);text-decoration:underline;text-underline-offset:3px}
code{
  font-family:"IBM Plex Mono",monospace;font-size:.86em;
  background:var(--surface-2);border:1px solid var(--line);
  border-radius:4px;padding:1px 5px;color:var(--ink)
}
hr{border:none;border-top:1px solid var(--line);margin:44px 0}

/* ---------- callout ---------- */
.callout{
  border:1px solid var(--line);border-left:3px solid var(--accent);
  background:var(--surface);border-radius:6px;
  padding:16px 20px;margin:18px 0;max-width:56em
}
.callout.teal{border-left-color:var(--teal)}
.callout .co-title{
  font-family:"IBM Plex Mono",monospace;font-size:12px;letter-spacing:.1em;
  text-transform:uppercase;color:var(--accent);margin-bottom:6px
}
.callout.teal .co-title{color:var(--teal)}
.callout p:last-child{margin-bottom:0}

/* ---------- terminal diagram ---------- */
.term{
  border:1px solid var(--line);border-radius:8px;overflow:hidden;
  margin:18px 0;background:var(--code-bg)
}
.term-bar{
  display:flex;align-items:center;gap:8px;padding:9px 14px;
  background:var(--code-line)
}
.term-bar .dot{width:10px;height:10px;border-radius:50%}
.dot.r{background:#e5726e}.dot.y{background:#e8c05a}.dot.g{background:#7bc47f}
.term-bar .term-title{
  font-family:"IBM Plex Mono",monospace;font-size:11.5px;color:var(--code-muted);
  margin-left:8px
}
.term-pre{
  overflow-x:auto;padding:18px 20px;
}
pre{
  font-family:"IBM Plex Mono",ui-monospace,monospace;
  font-size:13px;line-height:1.65;color:var(--code-ink);
  white-space:pre;tab-size:2
}
.term-pre pre{color:var(--code-ink)}
.hl-a{color:var(--code-amber)}
.hl-t{color:var(--code-teal)}
.hl-c{color:var(--code-muted)}

/* ---------- code block with file tab ---------- */
.codeblock{border:1px solid var(--line);border-radius:8px;overflow:hidden;margin:16px 0;background:var(--code-bg)}
.code-tab{
  font-family:"IBM Plex Mono",monospace;font-size:11.5px;color:var(--code-muted);
  background:var(--code-line);padding:7px 14px;display:flex;justify-content:space-between;gap:12px
}
.code-tab .lang{color:var(--code-teal)}
.code-scroll{overflow-x:auto;padding:16px 18px}
.k{color:#c586c0}
.f{color:#dcdcaa}
.s{color:#98c379}
.n{color:#d19a66}
.cm{color:var(--code-muted);font-style:italic}

/* ---------- tables ---------- */
.tbl-wrap{overflow-x:auto;margin:16px 0;border:1px solid var(--line);border-radius:8px}
table{border-collapse:collapse;width:100%;font-size:14px;background:var(--surface)}
th{
  font-family:"IBM Plex Mono",monospace;font-size:11.5px;letter-spacing:.08em;
  text-transform:uppercase;color:var(--muted);text-align:left;
  padding:10px 14px;border-bottom:1px solid var(--line);background:var(--surface-2)
}
td{padding:10px 14px;border-bottom:1px solid var(--line);vertical-align:top}
tr:last-child td{border-bottom:none}
td .mono, td.mono{font-size:12.5px;color:var(--ink)}
td.path{font-family:"IBM Plex Mono",monospace;font-size:12.5px;white-space:nowrap}

/* ---------- layer cards ---------- */
.layer{
  border:1px solid var(--line);border-radius:10px;background:var(--surface);
  margin:26px 0;overflow:hidden
}
.layer-head{
  display:flex;align-items:center;gap:16px;padding:18px 22px;
  border-bottom:1px solid var(--line);background:var(--surface-2)
}
.layer-badge{
  font-family:"IBM Plex Mono",monospace;font-weight:600;font-size:15px;
  color:var(--ground);background:var(--accent);border-radius:6px;
  padding:4px 11px;flex:0 0 auto
}
.layer-head h4{font-size:18px;font-weight:600;line-height:1.3}
.layer-head .est{
  margin-left:auto;font-family:"IBM Plex Mono",monospace;font-size:11.5px;
  color:var(--muted);flex:0 0 auto;white-space:nowrap
}
.layer-body{padding:20px 22px}
.goal{display:flex;gap:10px;margin-bottom:14px;max-width:none}
.goal .g-label{
  font-family:"IBM Plex Mono",monospace;font-size:11px;letter-spacing:.08em;
  color:var(--teal);background:var(--teal-soft);border-radius:4px;
  padding:2px 8px;height:fit-content;flex:0 0 auto;margin-top:3px
}
.goal p{margin:0}
.map{
  margin-top:16px;border-top:1px dashed var(--line);padding-top:12px;
  font-size:13px;color:var(--muted)
}
.map .mono{color:var(--ink-soft);font-size:12px}

/* ---------- step chain ---------- */
.chain{counter-reset:step;list-style:none;padding-left:0;margin:18px 0;max-width:none}
.chain li{
  counter-increment:step;position:relative;padding:0 0 18px 46px;max-width:52em
}
.chain li::before{
  content:counter(step,decimal-leading-zero);
  position:absolute;left:0;top:1px;
  font-family:"IBM Plex Mono",monospace;font-size:12px;color:var(--accent)
}
.chain li::after{
  content:"";position:absolute;left:11px;top:24px;bottom:2px;width:1px;background:var(--line)
}
.chain li:last-child::after{display:none}
.chain .who{font-family:"IBM Plex Mono",monospace;font-size:12px;color:var(--teal)}

footer{
  margin-top:72px;border-top:1px solid var(--line);
  padding:28px 0 60px;color:var(--muted);font-size:13px
}
@media (max-width:640px){
  header{padding-top:44px}
  .layer-head{flex-wrap:wrap}
  .layer-head .est{margin-left:0}
}
@media (prefers-reduced-motion: reduce){
  html{scroll-behavior:auto}
}
</style>
</head>
<body>

<header>
  <div class="wrap">
    <div class="eyebrow">Source Anatomy · 源码解剖手册</div>
    <h1>拆解 OpenCode：<br>一个开源 AI 编程智能体的全部思路</h1>
    <p class="lede">
      OpenCode 是 Claude Code 的开源对标产品：跑在终端里、能读写文件、执行命令、调用各家大模型的编程智能体。
      本文基于对当前仓库（Bun + TypeScript monorepo，30+ packages，核心约 15 万行）的逐层剖析，
      用最简单的方式讲清它的<strong>整体架构与核心实现</strong>，并给出一条<strong>从 0 到 1 复刻同功能的分层路径</strong>——
      每一层都附最小可运行的设计思路与 Demo 要点。
    </p>
    <div class="meta-row">
      <span class="meta-pill">runtime: Bun</span>
      <span class="meta-pill">framework: Effect (DI + 流)</span>
      <span class="meta-pill">LLM: Vercel AI SDK v6</span>
      <span class="meta-pill">UI: opentui + SolidJS</span>
      <span class="meta-pill">storage: git-backed files + SQLite</span>
    </div>
  </div>
</header>

<nav>
  <div class="wrap">
    <a href="#s1">01 一句话本质</a>
    <a href="#s2">02 架构总览</a>
    <a href="#s3">03 一次对话的旅程</a>
    <a href="#s4">04 十大核心机制</a>
    <a href="#s5">05 值得偷的设计</a>
    <a href="#s6">06 从 0 实现：L0–L9</a>
    <a href="#s7">07 源码阅读顺序</a>
  </div>
</nav>

<main class="wrap">

<!-- ================= 01 ================= -->
<section id="s1">
  <div class="sec-head"><span class="sec-num">01</span><h2>一句话本质</h2></div>

  <div class="callout">
    <div class="co-title">The Core</div>
    <p><strong>AI 编程智能体 = 一个 while 循环 + 一组工具 + 一个 LLM。</strong></p>
    <p>循环每一轮把对话历史发给模型；模型要么回答（结束），要么要求调用工具（执行工具 → 结果写回历史 → 继续循环）。
    OpenCode 其余的十几万行代码，全部是为了让这个循环<strong>更可靠、更安全、更可扩展、更好用</strong>。</p>
  </div>

  <div class="term">
    <div class="term-bar"><span class="dot r"></span><span class="dot y"></span><span class="dot g"></span><span class="term-title">agent-loop — 智能体的最小形态</span></div>
    <div class="term-pre"><pre><span class="hl-c">// 这就是 Claude Code / OpenCode / Cursor Agent 的共同骨架</span>
messages = [用户输入]
<span class="hl-a">while</span> true:
    response = <span class="hl-t">LLM</span>(系统提示 + messages + 工具定义)
    messages.append(response)
    <span class="hl-a">if</span> response 没有工具调用:
        <span class="hl-a">break</span>                      <span class="hl-c"># 模型给出最终回答，循环结束</span>
    <span class="hl-a">for</span> call <span class="hl-a">in</span> response.工具调用:
        result = <span class="hl-t">execute</span>(call)     <span class="hl-c"># 读文件 / 改代码 / 跑命令…</span>
        messages.append(result)    <span class="hl-c"># 结果回灌，下一轮模型能看到</span></pre></div>
  </div>

  <p>记住这张图，后面所有机制都是它的"挂件"：</p>
  <ul>
    <li><strong>工具系统</strong> — 决定 <code>execute()</code> 能干什么、怎么描述给模型；</li>
    <li><strong>权限系统</strong> — 决定 <code>execute()</code> 之前要不要问用户；</li>
    <li><strong>系统提示</strong> — 决定模型"是谁、在哪、有什么规矩"；</li>
    <li><strong>上下文管理</strong>（压缩/快照）— 决定 messages 太长、改错了怎么办；</li>
    <li><strong>服务器 + UI</strong> — 决定人怎么和这个循环交互、循环怎么被多个界面观察。</li>
  </ul>
</section>

<!-- ================= 02 ================= -->
<section id="s2">
  <div class="sec-head"><span class="sec-num">02</span><h2>架构总览：先看清两个关键决策</h2></div>

  <h3><span class="tag">决策 A</span>核心是一台服务器，所有 UI 都是客户端</h3>
  <p>OpenCode 最反直觉、也最值得学的一点：<strong>智能体运行时本身是一个 HTTP 服务器</strong>（Effect HttpApi 实现，
  带 OpenAPI 描述）。你平时看到的终端界面（TUI）只是它的一个客户端；Web 界面、桌面 App、
  <code>opencode run</code> 非交互模式、甚至第三方程序，全都通过<strong>自动生成的 SDK</strong> 调同一套 API、订阅同一条 SSE 事件流。</p>
  <p>这带来的好处：界面随便换、可以 <code>opencode attach</code> 连接到别处正在跑的会话、
  CI 里可以无界面运行、桌面版和终端版行为永远一致。</p>

  <h3><span class="tag">决策 B</span>严格的包依赖分层</h3>
  <p>monorepo 里的依赖方向被明文规定（见 AGENTS.md）：<code>schema → core / protocol → server → client → sdk</code>。
  纯数据包不许碰运行时，客户端不许碰服务端实现。这让"协议"成为系统的腰，上下都围着它转。</p>

  <div class="term">
    <div class="term-bar"><span class="dot r"></span><span class="dot y"></span><span class="dot g"></span><span class="term-title">architecture — 分层与数据流</span></div>
    <div class="term-pre"><pre>┌─────────────────────────── <span class="hl-t">客户端层（都只依赖 SDK）</span> ──────────────────────────┐
│  <span class="hl-t">tui</span> 终端界面        <span class="hl-t">app/desktop</span> 网页+桌面     <span class="hl-t">cli run</span> 非交互      第三方   │
│  opentui+SolidJS     SolidJS                一次性问答                     │
└────────────────────────────────┬─────────────────────────────────────┘
                                 │  HTTP + SSE（协议来自 OpenAPI，SDK 自动生成）
┌────────────────────────────────┴─────────────────────────────────────┐
│ <span class="hl-a">protocol</span> 定义 API 形状 ──► <span class="hl-a">server</span> 实现 handlers ──► <span class="hl-a">client</span> 生成 SDK    │
│                                                                      │
│  <span class="hl-a">opencode</span>（主包：智能体运行时）                                        │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────┐ ┌───────────────┐  │
│  │ SessionPrompt │→│  Processor   │→│ Permission │ │  Snapshot     │  │
│  │ <span class="hl-c">主循环 while</span>  │ │ <span class="hl-c">流事件→消息</span>   │ │ <span class="hl-c">allow/ask</span>  │ │ <span class="hl-c">git 快照回滚</span>  │  │
│  └──────┬───────┘ └──────┬───────┘ └────────────┘ └───────────────┘  │
│  ┌──────┴───────┐ ┌──────┴───────┐ ┌────────────┐ ┌───────────────┐  │
│  │ Agent/系统提示 │ │ <span class="hl-t">LLM 抽象层</span>    │ │ 工具注册表   │ │ MCP/插件/技能  │  │
│  │ AGENTS.md 等  │ │ AI SDK+原生  │ │ read/edit… │ │ 外部扩展       │  │
│  └──────────────┘ └──────────────┘ └────────────┘ └───────────────┘  │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 持久层：会话/消息/部件（V1: git 文件存储 · V2: SQLite+drizzle）      │  │
│  │ 事件总线：进程内 EventEmitter → SSE 推给所有客户端                  │  │
│  └────────────────────────────────────────────────────────────────┘  │
│  <span class="hl-a">core</span>：领域服务（配置/模型目录/凭据/文件系统/权限/会话 V2/PTY…）          │
│  <span class="hl-a">schema</span>：纯数据类型（Effect Schema），零运行时依赖                      │
└──────────────────────────────────────────────────────────────────────┘</pre></div>
  </div>

  <h3><span class="tag">地图</span>关键 package 职责速查</h3>
  <div class="tbl-wrap">
    <table>
      <thead><tr><th>包</th><th>职责</th><th>一句话</th></tr></thead>
      <tbody>
        <tr><td class="path">schema</td><td>全部领域数据类型</td><td>消息、会话、事件、权限…的 Effect Schema 定义，纯叶子</td></tr>
        <tr><td class="path">core</td><td>领域服务</td><td>配置、模型目录(models.dev)、凭据/OAuth、文件系统、SQLite 会话(V2)、PTY、ripgrep 封装</td></tr>
        <tr><td class="path">protocol</td><td>HTTP API 定义</td><td>按资源分组的端点：session / message / agent / model / permission / event / pty…</td></tr>
        <tr><td class="path">server</td><td>API 实现</td><td>handlers + 鉴权/定位中间件，输出 OpenAPI</td></tr>
        <tr><td class="path">client</td><td>生成的 SDK</td><td>Promise 版 + Effect 版客户端，浏览器安全</td></tr>
        <tr><td class="path">opencode</td><td>CLI 主包</td><td>主循环、全部内置工具、provider 接入、TUI 命令入口（<code>yargs</code>）</td></tr>
        <tr><td class="path">llm</td><td>LLM 协议层</td><td>自研 provider 协议适配器 + 统一事件模型 <code>LLMEvent</code></td></tr>
        <tr><td class="path">tui</td><td>终端界面</td><td>opentui（自研终端渲染引擎）+ SolidJS 响应式</td></tr>
        <tr><td class="path">app / desktop</td><td>网页 / 桌面</td><td>SolidJS 应用，与 TUI 共享 UI 组件</td></tr>
        <tr><td class="path">plugin</td><td>插件 API 类型</td><td>钩子定义：tool.execute.before/after、chat.params…</td></tr>
        <tr><td class="path">sdk / sdk-next</td><td>嵌入式宿主</td><td>把整台"服务器"以内存方式嵌进你的进程</td></tr>
      </tbody>
    </table>
  </div>
</section>

<!-- ================= 03 ================= -->
<section id="s3">
  <div class="sec-head"><span class="sec-num">03</span><h2>一次对话的完整旅程</h2></div>
  <p>把"你在终端里输入一句话，AI 改了三个文件并回答你"这件事拆开，整条链路如下。读懂这条链，就读懂了 OpenCode 的 70%。</p>

  <ol class="chain">
    <li><span class="who">TUI</span> 你敲下"修复 app.ts 里的 bug"回车。TUI 通过 SDK 发出 <code>POST /session/:id/prompt</code>，请求体里是消息部件（文本/文件引用/@的 agent）。</li>
    <li><span class="who">server → SessionPrompt.prompt()</span> 服务端把用户消息写入存储（消息 + parts 模型），立刻发布 <code>message.updated</code> 事件 → SSE 推到 TUI，用户消息即刻上屏。</li>
    <li><span class="who">runLoop()</span> 进入 <code>while(true)</code> 主循环：读取历史消息（跳过已压缩的），检查上一条助手消息是否"完结且无待执行工具"——是则退出循环。</li>
    <li><span class="who">组装请求</span> 解析本条消息选定的 agent（build/plan）与模型；拼系统提示 = 模型家族模板 + 环境信息 + AGENTS.md 指令 + 技能清单 + MCP 说明；解析该 agent 可用的工具集（内置 + MCP + 插件）。</li>
    <li><span class="who">LLM.stream()</span> 调 Vercel AI SDK 的 <code>streamText</code>（或实验性自研原生运行时），得到统一的 <code>LLMEvent</code> 事件流：text-delta / reasoning-delta / tool-call / tool-result / finish…</li>
    <li><span class="who">Processor</span> 逐个消费事件：文本增量写入 TextPart、推理写入 ReasoningPart、工具调用写入 ToolPart——每次写入都发事件，TUI 因此获得打字机效果的实时渲染。执行工具前先过权限检查，并打一个 git 快照（可撤销）。</li>
    <li><span class="who">工具执行</span> 比如 <code>read</code> 读文件（带行数/字节上限与截断）、<code>edit</code> 精确替换、<code>bash</code> 跑命令。输出过大自动截断并把全文存到托管文件；参数不合法时把错误信息作为工具结果喂回给模型让它重写。</li>
    <li><span class="who">回到循环顶端</span> 模型看到工具结果，可能继续调工具，也可能给出最终回答。期间若 token 逼近上限 → 自动触发压缩（让小模型总结历史）；同一工具重复调用 3 次 → doom-loop 检测介入。</li>
    <li><span class="who">收尾</span> 助手消息标记 finish、统计 token 与花费、会话回到 idle 状态；TUI 收到状态事件，输入框重新可用。整个会话历史已持久化，可随时 <code>--continue</code> 恢复或 fork。</li>
  </ol>
</section>

<!-- ================= 04 ================= -->
<section id="s4">
  <div class="sec-head"><span class="sec-num">04</span><h2>十大核心机制详解</h2></div>

  <h3><span class="tag">M1</span>主循环 — <code>session/prompt.ts</code> 的 runLoop</h3>
  <p>全项目最重要的 200 行。剥掉所有分支后的骨架：</p>
  <div class="codeblock">
    <div class="code-tab"><span>packages/opencode/src/session/prompt.ts（简化骨架）</span><span class="lang">TypeScript</span></div>
    <div class="code-scroll"><pre><span class="k">while</span> (<span class="k">true</span>) {
  msgs = 读取历史消息(跳过已压缩)
  { lastUser, lastAssistant } = 找到最近的用户消息与助手消息

  <span class="cm">// 退出条件：助手已完结(stop/length…) 且没有待执行的工具调用</span>
  <span class="k">if</span> (lastAssistant.finish && lastAssistant.finish !== <span class="s">"tool-calls"</span> && !hasToolCalls)
    <span class="k">break</span>

  <span class="cm">// 待处理的特殊任务：子代理(subtask) 或 压缩(compaction)</span>
  <span class="k">if</span> (task?.type === <span class="s">"subtask"</span>)    { handleSubtask(...); <span class="k">continue</span> }
  <span class="k">if</span> (task?.type === <span class="s">"compaction"</span>) { compaction.process(...); <span class="k">continue</span> }

  <span class="cm">// 上下文快溢出 → 自动安排一次压缩，然后继续</span>
  <span class="k">if</span> (上一条消息 token 超限) { compaction.create(auto); <span class="k">continue</span> }

  agent = 取出本条消息指定的 agent      <span class="cm">// build / plan / 自定义</span>
  step++;  <span class="k">if</span> (step >= agent.steps) 注入 <span class="s">"请收尾"</span> 提示   <span class="cm">// 步数上限</span>

  tools  = 解析该 agent 可用工具(内置 + MCP + 插件 + 权限过滤)
  system = 环境信息 + AGENTS.md 指令 + 技能清单 + MCP 说明

  result = processor.process({ system, messages, tools, model })
  <span class="cm">// process 内部消费 LLM 流、执行工具、落库</span>
  <span class="cm">// 返回 "stop" → break；"compact" → 安排压缩；否则继续循环</span>
}</pre></div>
  </div>
  <div class="callout teal">
    <div class="co-title">为什么这样写</div>
    <p>循环的"退出条件"放在<strong>顶部</strong>而不是底部，是因为工具结果由我们自己写入历史后 <code>continue</code> 回顶部重新判断——
    这让压缩、子任务、中断恢复都能以"往历史里写一条消息"的方式统一接入。循环本身无状态，状态全在消息历史里，
    崩溃后靠重放历史即可恢复。</p>
  </div>

  <h3><span class="tag">M2</span>工具系统 — <code>tool/tool.ts</code> 的 Tool.define</h3>
  <p>每个工具 = <strong>参数 Schema + 描述文本 + execute 函数</strong>，用 <code>Tool.define("read", …)</code> 注册。三个关键设计：</p>
  <ul>
    <li><strong>描述与代码分离</strong>：工具的 LLM 说明放在同名 <code>.txt</code> 文件（<code>read.txt</code>、<code>edit.txt</code>…），改提示词不用碰逻辑，且以字符串导入打包。</li>
    <li><strong>参数校验失败的反馈闭环</strong>：模型传错参数时抛 <code>InvalidArgumentsError</code>，其 message 会作为工具结果喂回模型——"请重写输入使其满足 schema"——模型通常一次就能改对。</li>
    <li><strong>输出统一治理</strong>：所有工具输出经过同一个截断层（行数/字节双上限，保头保尾），超限全文转存"托管输出文件"，历史里只留有界预览——防止一次 <code>bash</code> 打爆上下文。</li>
  </ul>
  <div class="codeblock">
    <div class="code-tab"><span>packages/opencode/src/tool/tool.ts（接口本质）</span><span class="lang">TypeScript</span></div>
    <div class="code-scroll"><pre><span class="k">interface</span> <span class="f">ToolDef</span> {
  id: <span class="k">string</span>
  description: <span class="k">string</span>                <span class="cm">// 给模型看的说明书（.txt 导入）</span>
  parameters: Schema               <span class="cm">// Effect Schema → 自动转 JSON Schema 给模型</span>
  execute(args, ctx): Effect&lt;{
    title: <span class="k">string</span>                    <span class="cm">// UI 上的一行标题，如 "read src/app.ts"</span>
    metadata: <span class="k">object</span>                 <span class="cm">// UI 展示用的结构化数据</span>
    output: <span class="k">string</span>                   <span class="cm">// 喂回给模型的文本</span>
  }&gt;
}
<span class="cm">// ctx 里带：sessionID、AbortSignal、messages、</span>
<span class="cm">// metadata()（执行中实时更新 UI）、ask()（请求权限）</span></pre></div>
  </div>
  <p>内置工具全家桶：<code>read / write / edit / glob / grep / bash(shell) / task(子代理) / webfetch / websearch / todowrite / question(反问用户) / lsp / skill / plan</code>。
  细节值得逐个读：<code>read</code> 找不到文件会 fuzzy 推荐相似文件名；<code>edit</code> 要求 old_string 唯一匹配否则报错；<code>bash</code> 每会话维护持久 shell 状态。</p>

  <h3><span class="tag">M3</span>LLM 抽象层 — <code>session/llm.ts</code> + <code>packages/llm</code></h3>
  <p>默认运行时是 Vercel <strong>AI SDK</strong>（<code>streamText</code>），一套适配器接住所有 provider；同时预留自研原生运行时
  （<code>packages/llm</code>，直连各家协议），两条路都输出<strong>同一种事件流 <code>LLMEvent</code></strong>——上层 Processor 只认事件，不认 provider。
  模型目录来自 <strong>models.dev</strong>（开源模型数据库），本地缓存 + 定时更新，provider/model 的定价、能力、API 形状全在里面。</p>
  <p>两个精彩的小处理：</p>
  <ul>
    <li><code>experimental_repairToolCall</code>：模型把工具名写错大小写 → 自动修正；实在修不了 → 路由到一个叫 <code>invalid</code> 的兜底工具，把错误讲给模型听。</li>
    <li>模型中间件 <code>wrapLanguageModel</code>：在请求出门前按 provider 做消息变换（如 reasoning 块的处理、缓存断点），provider 差异被隔离在这一层。</li>
  </ul>

  <h3><span class="tag">M4</span>系统提示的组装 — <code>session/system.ts</code> + <code>session/instruction.ts</code></h3>
  <p>系统提示不是一个大字符串，而是<strong>按模型家族选模板 + 动态拼装</strong>：</p>
  <div class="term">
    <div class="term-bar"><span class="dot r"></span><span class="dot y"></span><span class="dot g"></span><span class="term-title">system prompt 的配方</span></div>
    <div class="term-pre"><pre>① 人格模板（按模型挑）  claude → anthropic.txt · gpt → gpt.txt · gemini → gemini.txt …
② 环境信息            模型名、工作目录、仓库根、是否 git、平台、日期
③ AGENTS.md 指令      从当前目录向上逐级收集 + 全局配置（可热更新）
④ 技能清单            只给名字+描述，正文要通过 skill 工具按需读取（省 token）
⑤ MCP 服务器说明       各 MCP server 自带的 instructions
⑥ 结构化输出要求       当用户要求 JSON schema 输出时追加</pre></div>
  </div>

  <h3><span class="tag">M5</span>权限系统 — <code>permission/index.ts</code></h3>
  <p>模型要跑 <code>rm -rf</code>？权限层是最后防线。规则极简：<code>{ permission, pattern, action }</code>，action ∈ <code>allow | ask | deny</code>，
  匹配用通配符，多条规则<strong>后者覆盖前者</strong>。规则来源按优先级合并：全局配置 → 项目配置 → agent 定义 → 会话内用户选择。</p>
  <p><code>ask</code> 的实现是精髓：工具执行到一个 <code>Effect.Deferred</code> 上<strong>阻塞</strong>，同时发出 <code>permission.asked</code> 事件 →
  UI 弹出确认框 → 用户点允许/拒绝/永久允许 → 服务端 <code>reply()</code> 解锁 Deferred。整个过程工具代码毫无感知，就像同步等待。</p>
  <p>内置两个 agent 体现了权限的用途：<strong>build</strong>（默认，全权限）与 <strong>plan</strong>（只读：禁文件编辑、bash 需询问）——同一个循环，不同的规则集。</p>

  <h3><span class="tag">M6</span>消息模型与持久化</h3>
  <p>一条 Message 不是纯文本，而是<strong>部件(parts)的集合</strong>：text / reasoning / tool / file / step-start / subtask…
  流式写入时每个 part 独立更新，UI 想渲染什么取什么。存储有两代并行：</p>
  <ul>
    <li><strong>V1</strong>（现役主路径）：文件存储 + <strong>用 git 做存储引擎</strong>——会话目录本身是个 git 仓库，每次写入即提交，天然获得历史与一致性；</li>
    <li><strong>V2</strong>（演进中）：SQLite + drizzle + Effect，"可靠收件箱"式设计——用户输入先持久化为 inbox 行，再由串行运行器在安全边界提升为可见消息，崩溃可重放。</li>
  </ul>
  <p>事件侧：进程内 EventEmitter 总线（每个事件自动分配递增 ID）→ 服务器以 SSE 暴露两条流：
  <strong>实例级实时流</strong>（<code>events.subscribe</code>，无重放保证）与<strong>会话级持久流</strong>（<code>sessions.events(after)</code>，按序号可断点续传）。</p>

  <h3><span class="tag">M7</span>Snapshot：git 快照实现"撤销 AI"</h3>
  <p>每轮模型动手前，在一个<strong>隐藏的旁路 git 仓库</strong>里对工作区打快照（只跟踪、不影响你项目的 git）。
  于是什么都能算出来：这轮改了哪些文件（patch）、恢复到某步之前（restore）、撤销最近几轮（revert）。
  UI 里的"undo AI 修改"就是 <code>git revert</code> 的包装——几乎零成本获得安全网。</p>

  <h3><span class="tag">M8</span>上下文压缩（Compaction）</h3>
  <p>token 逼近模型上限时自动触发：用一个专门的 compaction agent（小模型+总结提示）把旧历史压成一段摘要消息，
  旧消息标记为 compacted 退出后续请求，但<strong>不删除</strong>——UI 还能看全历史，只是模型看不到了。
  压缩本身也作为一条消息写进历史，所以它天然融入循环（见 M1 里的 <code>task.type === "compaction"</code>）。</p>

  <h3><span class="tag">M9</span>子代理（Subagents）— task 工具</h3>
  <p>主模型可以调用 <code>task</code> 工具派生子任务：选择一个 agent（如内置的 <code>general</code> 搜索代理、<code>explore</code>），
  实际上是<strong>创建一个子会话并递归跑同一个主循环</strong>，跑完把结果作为工具输出返回。
  子代理有独立上下文（不污染主会话）、独立权限、可指定不同模型——这就是"多代理"的全部秘密：递归 + 隔离。</p>

  <h3><span class="tag">M10</span>扩展体系：Plugin / MCP / Skill / Command</h3>
  <div class="tbl-wrap">
    <table>
      <thead><tr><th>机制</th><th>形态</th><th>能力</th></tr></thead>
      <tbody>
        <tr><td><strong>Plugin</strong></td><td>JS/TS 模块（本地文件/npm/内联），配置声明加载</td><td>生命周期钩子：<code>tool.execute.before/after</code>、修改请求参数、变换系统提示、注册自定义工具</td></tr>
        <tr><td><strong>MCP</strong></td><td>外部进程（stdio/SSE），标准协议</td><td>动态接入第三方工具与资源，带 OAuth；工具经同一注册表与权限层</td></tr>
        <tr><td><strong>Skill</strong></td><td>带 frontmatter 的 markdown 文件</td><td>按需加载的"能力说明书"，系统提示只放清单，正文经 skill 工具读取</td></tr>
        <tr><td><strong>Command</strong></td><td>markdown 模板（<code>/xxx</code> 斜杠命令）</td><td>参数占位符替换后作为用户消息注入，可指定 agent</td></tr>
      </tbody>
    </table>
  </div>
</section>

<!-- ================= 05 ================= -->
<section id="s5">
  <div class="sec-head"><span class="sec-num">05</span><h2>值得偷的 12 个设计决策</h2></div>
  <ol>
    <li><strong>工具错误是"对话"不是"崩溃"</strong> — 参数校验失败、文件找不到，都变成喂回模型的文本，让模型自我纠正；循环几乎不因工具失败而中断。</li>
    <li><strong>一切状态在消息历史里</strong> — 主循环无状态，崩溃恢复、fork、分享、导出全部变成"操作历史"。</li>
    <li><strong>UI 只是客户端</strong> — 先有 API 再有界面；于是 attach、web 版、桌面版、CI 模式几乎免费。</li>
    <li><strong>协议即腰</strong> — OpenAPI 定义 → 自动生成 SDK → 类型从服务端一路贯通到 UI，永不手滑。</li>
    <li><strong>事件流分"实时"与"持久"两种</strong> — 实时流轻量无保证；持久流带序号可断点续传。不混用。</li>
    <li><strong>提示词工程产品化</strong> — 按模型家族分模板、txt 与代码分离、指令(AGENTS.md)热加载，提示词像配置一样管理。</li>
    <li><strong>权限 = 数据不是代码</strong> — <code>{permission, pattern, action}</code> 三条目规则集，可在配置、agent、会话三层叠加，UI 只需一个确认框。</li>
    <li><strong>用 git 解决一切文件问题</strong> — 存储引擎是 git，撤销是 git，diff 是 git。不重复造轮子。</li>
    <li><strong>输出治理前置</strong> — 所有工具输出统一截断 + 全文托管，上下文预算不被单个工具打爆。</li>
    <li><strong>Doom loop 检测</strong> — 同一工具同样参数连续 3 次即介入，防止模型原地打转烧钱。</li>
    <li><strong>技能"目录与正文分离"</strong> — 系统提示只放清单，正文按需读取，几十个技能也不占常驻 token。</li>
    <li><strong>子代理 = 递归调用自己</strong> — 不引入新的编排框架，一个 task 工具 + 子会话解决多代理。</li>
  </ol>
</section>

<!-- ================= 06 ================= -->
<section id="s6">
  <div class="sec-head"><span class="sec-num">06</span><h2>从 0 实现：十层进阶路径</h2></div>
  <p>下面的路径<strong>从底层到应用层层层递进</strong>，每一层都可独立交付、独立验证，括号里是预估投入。
  走完 L0–L4 你就有一个能用的"Claude Code 内核"；L5–L9 是产品化与生态。技术栈建议与 OpenCode 一致：
  <strong>TypeScript + Bun</strong>（你也可以用 Python，思路完全相同）。</p>

  <div class="term">
    <div class="term-bar"><span class="dot r"></span><span class="dot y"></span><span class="dot g"></span><span class="term-title">roadmap — 层层递进</span></div>
    <div class="term-pre"><pre>L0 裸循环 ──► L1 工具框架 ──► L2 消息与持久化 ──► L3 系统提示 ──► L4 权限
                                                                     │
L9 生态(子代理/插件/MCP) ◄── L8 UI(TUI/Web) ◄── L7 服务化(HTTP+SSE+SDK) ◄┘
                                     ▲
                    L6 多 Provider 抽象 ◄── L5 会话增强(压缩/快照/fork)</pre></div>
  </div>

  <!-- L0 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L0</span><h4>裸智能体循环 — 100 行跑通一切的原点</h4><span class="est">≈ 半天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>单个 bash 工具 + 直接 fetch 模型 API，跑通"提问 → 调工具 → 回答"。这一层你会彻底看懂所有 agent 产品的心脏。</p></div>
      <div class="codeblock">
        <div class="code-tab"><span>my-agent/agent.ts — bun agent.ts "统计当前目录代码行数"</span><span class="lang">TypeScript</span></div>
        <div class="code-scroll"><pre><span class="k">import</span> { execFileSync } <span class="k">from</span> <span class="s">"child_process"</span>

<span class="k">const</span> SYSTEM = <span class="s">`你是一个编程助手，可用 bash 工具操作当前目录。
工作目录: ${process.cwd()}。先调查再动手，回答简洁。`</span>

<span class="k">const</span> TOOLS = [{
  name: <span class="s">"bash"</span>,
  description: <span class="s">"执行 shell 命令，返回 stdout+stderr"</span>,
  input_schema: {
    type: <span class="s">"object"</span>,
    properties: { command: { type: <span class="s">"string"</span>, description: <span class="s">"要执行的命令"</span> } },
    required: [<span class="s">"command"</span>],
  },
}]

<span class="k">function</span> <span class="f">callLLM</span>(messages: <span class="k">any</span>[]) {
  <span class="k">return</span> fetch(<span class="s">"https://api.anthropic.com/v1/messages"</span>, {
    method: <span class="s">"POST"</span>,
    headers: {
      <span class="s">"x-api-key"</span>: process.env.ANTHROPIC_API_KEY!,
      <span class="s">"anthropic-version"</span>: <span class="s">"2023-06-01"</span>,
      <span class="s">"content-type"</span>: <span class="s">"application/json"</span>,
    },
    body: JSON.stringify({
      model: <span class="s">"claude-sonnet-4-5"</span>, max_tokens: <span class="n">8192</span>,
      system: SYSTEM, tools: TOOLS, messages,
    }),
  }).then(r =&gt; r.json())
}

<span class="k">function</span> <span class="f">execTool</span>(name: <span class="k">string</span>, input: <span class="k">any</span>): <span class="k">string</span> {
  <span class="k">if</span> (name !== <span class="s">"bash"</span>) <span class="k">return</span> <span class="s">`未知工具: ${name}`</span>
  <span class="k">try</span> {  <span class="cm">// ← 错误变成文本喂回模型，而不是崩溃（OpenCode 同款思路）</span>
    <span class="k">return</span> execFileSync(<span class="s">"bash"</span>, [<span class="s">"-c"</span>, input.command],
      { encoding: <span class="s">"utf8"</span>, timeout: <span class="n">60_000</span>, maxBuffer: <span class="n">1e6</span> })
  } <span class="k">catch</span> (e: <span class="k">any</span>) {
    <span class="k">return</span> <span class="s">`exit ${e.status}\n${e.stdout ?? ""}${e.stderr ?? e.message}`</span>
  }
}

<span class="k">async function</span> <span class="f">main</span>() {
  <span class="k">const</span> messages: <span class="k">any</span>[] = [{ role: <span class="s">"user"</span>, content: process.argv[<span class="n">2</span>] }]
  <span class="k">while</span> (<span class="k">true</span>) {                          <span class="cm">// ← 主循环！</span>
    <span class="k">const</span> res = <span class="k">await</span> callLLM(messages)
    messages.push({ role: <span class="s">"assistant"</span>, content: res.content })
    <span class="k">const</span> calls = res.content.filter((b: <span class="k">any</span>) =&gt; b.type === <span class="s">"tool_use"</span>)
    <span class="k">if</span> (res.stop_reason !== <span class="s">"tool_use"</span> || calls.length === <span class="n">0</span>) {
      console.log(res.content.filter((b: <span class="k">any</span>) =&gt; b.type === <span class="s">"text"</span>)
        .map((b: <span class="k">any</span>) =&gt; b.text).join(<span class="s">"\n"</span>))
      <span class="k">return</span>                              <span class="cm">// ← 没有工具调用 = 结束</span>
    }
    messages.push({                       <span class="cm">// ← 工具结果回灌</span>
      role: <span class="s">"user"</span>,
      content: calls.map((c: <span class="k">any</span>) =&gt; ({
        type: <span class="s">"tool_result"</span>, tool_use_id: c.id,
        content: execTool(c.name, c.input).slice(<span class="n">0</span>, <span class="n">50_000</span>),
      })),
    })
  }
}
main()</pre></div>
      </div>
      <p class="tight">验收标准：它能自己 <code>ls</code> → <code>wc -l</code> → 汇报结果。此刻你已经复刻了 OpenCode 的 <code>runLoop</code> 本质。</p>
      <div class="map">对应 OpenCode：<span class="mono">session/prompt.ts (runLoop)</span> · <span class="mono">session/llm.ts (callLLM)</span> · <span class="mono">tool/shell.ts (bash)</span></div>
    </div>
  </div>

  <!-- L1 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L1</span><h4>工具框架 — 注册表、Schema 校验、错误闭环</h4><span class="est">≈ 1–2 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>把"工具"从硬编码升级为声明式插件：每个工具自描述（名字/说明/参数 schema/执行器），新增工具只需注册一行。</p></div>
      <div class="codeblock">
        <div class="code-tab"><span>tools.ts — 最小工具注册表</span><span class="lang">TypeScript</span></div>
        <div class="code-scroll"><pre><span class="k">import</span> { z } <span class="k">from</span> <span class="s">"zod"</span>   <span class="cm">// 或 OpenCode 用的 Effect Schema / valibot</span>

<span class="k">export interface</span> <span class="f">Tool</span>&lt;T&gt; {
  name: <span class="k">string</span>
  description: <span class="k">string</span>
  schema: z.ZodType&lt;T&gt;
  run(input: T): Promise&lt;<span class="k">string</span>&gt;
}

<span class="k">export const</span> registry = <span class="k">new</span> Map&lt;<span class="k">string</span>, Tool&lt;<span class="k">any</span>&gt;&gt;()
<span class="k">export const</span> <span class="f">defineTool</span> = &lt;T,&gt;(t: Tool&lt;T&gt;) =&gt; registry.set(t.name, t)

defineTool({ name: <span class="s">"read"</span>, description: <span class="s">"读文件(带行号)"</span>,
  schema: z.object({ path: z.string(), offset: z.number().optional() }),
  run: <span class="k">async</span> ({path, offset}) =&gt; { <span class="cm">/* 读取 + 行号 + 截断 */</span> } })
defineTool({ name: <span class="s">"edit"</span>, <span class="cm">/* old_string 必须唯一匹配，否则报错让模型重写 */</span> })
defineTool({ name: <span class="s">"glob"</span>, <span class="cm">/* 文件名模式搜索 */</span> })
defineTool({ name: <span class="s">"grep"</span>, <span class="cm">/* 内容搜索，可直接调 ripgrep */</span> })

<span class="cm">// 主循环中的执行器：校验 + 错误回喂 + 输出截断</span>
<span class="k">export async function</span> <span class="f">executeTool</span>(name: <span class="k">string</span>, raw: <span class="k">unknown</span>) {
  <span class="k">const</span> tool = registry.get(name)
  <span class="k">if</span> (!tool) <span class="k">return</span> <span class="s">`工具 ${name} 不存在，可用: ${[...registry.keys()].join(", ")}`</span>
  <span class="k">const</span> parsed = tool.schema.safeParse(raw)
  <span class="k">if</span> (!parsed.success)
    <span class="k">return</span> <span class="s">`参数不合法: ${parsed.error.message}。请按 schema 重写输入。`</span> <span class="cm">// ← 回喂模型</span>
  <span class="k">try</span> { <span class="k">return</span> truncate(<span class="k">await</span> tool.run(parsed.data), <span class="n">50_000</span>) }
  <span class="k">catch</span> (e: <span class="k">any</span>) { <span class="k">return</span> <span class="s">`执行失败: ${e.message}`</span> }
}

<span class="cm">// 给 API 的工具定义自动从 zod 生成 JSON Schema</span>
<span class="k">export const</span> apiTools = [...registry.values()].map(t =&gt; ({
  name: t.name, description: t.description,
  input_schema: zodToJsonSchema(t.schema),
}))</pre></div>
      </div>
      <p class="tight">建议工具集按 OpenCode 的顺序实现：<code>read → write → edit → glob → grep → bash</code>。
      每个工具的<strong>描述文本</strong>值得直接从 OpenCode 的 <code>*.txt</code> 借鉴——那是被海量真实使用打磨过的提示词。</p>
      <div class="map">对应 OpenCode：<span class="mono">tool/tool.ts (define)</span> · <span class="mono">tool/registry.ts</span> · <span class="mono">tool/*.txt</span> · <span class="mono">tool/truncate.ts</span></div>
    </div>
  </div>

  <!-- L2 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L2</span><h4>消息模型与持久化 — 会话可恢复、可观察</h4><span class="est">≈ 2 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>引入 Message/Part 数据模型与存储，支持多会话、重启恢复。这是从"脚本"变成"产品"的分水岭。</p></div>
      <ul>
        <li><strong>数据模型</strong>：<code>Session { id, title, createdAt }</code>；<code>Message { id, sessionID, role, parts[] }</code>；
        <code>Part</code> 是联合类型：<code>text | reasoning | tool { name, state: pending/running/completed/error, input, output } | file</code>。
        关键洞察：<strong>流式过程 = 不断更新 part</strong>，UI 渲染的是 part 而不是字符串。</li>
        <li><strong>存储</strong>：起步用 JSON 文件（<code>~/.myagent/storage/session/&lt;id&gt;/info.json + messages/*.json</code>）即可；
        进阶可学 OpenCode 用 git 管这个目录（白拿版本历史），或直接 SQLite。</li>
        <li><strong>ID 策略</strong>：OpenCode 用 ULID（时间有序），排序即时间序，值得照抄。</li>
      </ul>
      <div class="codeblock">
        <div class="code-tab"><span>storage.ts — 关键接口（实现随意，接口要稳）</span><span class="lang">TypeScript</span></div>
        <div class="code-scroll"><pre><span class="k">interface</span> <span class="f">Storage</span> {
  sessionCreate(): Session
  sessionList(): Session[]
  sessionGet(id: <span class="k">string</span>): Session
  messageAppend(sessionID: <span class="k">string</span>, msg: Message): <span class="k">void</span>
  messageUpdate(sessionID: <span class="k">string</span>, msg: Message): <span class="k">void</span>   <span class="cm">// 流式更新 part</span>
  messages(sessionID: <span class="k">string</span>): Message[]
}
<span class="cm">// 主循环改造：每产生/更新一条消息就落盘 + 调 onChange(msg) 通知观察者</span>
<span class="cm">// —— 这个 onChange 就是未来 SSE 事件流的源头</span></pre></div>
      </div>
      <div class="map">对应 OpenCode：<span class="mono">session/message-v2.ts (模型)</span> · <span class="mono">storage/storage.ts (V1 git 存储)</span> · <span class="mono">core/database (V2 SQLite)</span></div>
    </div>
  </div>

  <!-- L3 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L3</span><h4>系统提示工程 — 让模型知道"它是谁、在哪、守什么规矩"</h4><span class="est">≈ 1 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>把系统提示从一句话变成组装流水线，并支持项目级指令文件。</p></div>
      <ul>
        <li>模板化：人格模板 + <code>&lt;env&gt;</code> 环境块（cwd / 平台 / 日期 / git 状态）+ 工具使用守则；</li>
        <li><strong>AGENTS.md 机制</strong>：从当前目录向上逐级查找 <code>AGENTS.md</code>（加全局 <code>~/.config/myagent/AGENTS.md</code>），
        拼接注入。这是让智能体"懂你的项目"的最低成本方案；</li>
        <li>进阶（OpenCode V2 的做法）：把每类上下文做成独立的"Context Source"（键 + 加载器 + 渲染器），
        在每次调模型前的安全边界统一比对变化——变更以"对话中的系统消息"形式进入历史，既省缓存又不会丢更新。</li>
      </ul>
      <div class="map">对应 OpenCode：<span class="mono">session/system.ts</span> · <span class="mono">session/prompt/*.txt</span> · <span class="mono">session/instruction.ts</span> · <span class="mono">core/system-context/</span></div>
    </div>
  </div>

  <!-- L4 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L4</span><h4>权限与安全 — 从玩具到敢交给别人用</h4><span class="est">≈ 2 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>危险操作执行前拦下来问人；规则可配置、可记忆。</p></div>
      <div class="codeblock">
        <div class="code-tab"><span>permission.ts — OpenCode 规则模型的极简复刻</span><span class="lang">TypeScript</span></div>
        <div class="code-scroll"><pre><span class="k">type</span> Rule = { permission: <span class="k">string</span>; pattern: <span class="k">string</span>; action: <span class="s">"allow"</span>|<span class="s">"ask"</span>|<span class="s">"deny"</span> }
<span class="cm">// 例: { permission: "edit",  pattern: "*.config.*", action: "ask"  }</span>
<span class="cm">//     { permission: "bash",  pattern: "rm *",       action: "deny" }</span>
<span class="cm">//     { permission: "bash",  pattern: "git *",      action: "allow"}</span>

<span class="k">function</span> <span class="f">evaluate</span>(permission: <span class="k">string</span>, pattern: <span class="k">string</span>, rules: Rule[]) {
  <span class="cm">// 通配符匹配，后写的规则覆盖先写的；没命中默认 ask</span>
  <span class="k">return</span> rules.findLast(r =&gt; wildcard(r.permission, permission)
    &amp;&amp; wildcard(r.pattern, pattern))?.action ?? <span class="s">"ask"</span>
}

<span class="cm">// ask 的实现：返回一个 Promise，只有用户在终端确认后才 resolve</span>
<span class="k">async function</span> <span class="f">askUser</span>(tool: <span class="k">string</span>, input: <span class="k">any</span>): Promise&lt;<span class="k">boolean</span>&gt; {
  <span class="cm">/* readline: "允许 edit src/app.ts 吗? [y/n/a(永久)]" */</span>
}</pre></div>
      </div>
      <p class="tight">规则来源分层合并（配置 → agent → 会话内记忆），并加一道静态防线：
      bash 命令先做简单解析，<code>rm -rf /</code>、写 <code>~/.ssh</code> 之类直接拒。OpenCode 还有 <code>plan</code> 只读 agent 可参考。</p>
      <div class="map">对应 OpenCode：<span class="mono">permission/index.ts + evaluate.ts</span> · <span class="mono">agent/agent.ts (build/plan 定义)</span></div>
    </div>
  </div>

  <!-- L5 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L5</span><h4>会话增强 — 压缩、快照、fork</h4><span class="est">≈ 3–4 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>长对话不爆上下文、改错可撤销、历史可分叉。</p></div>
      <ul>
        <li><strong>压缩</strong>：估算 token（按 usage 或字符数），超过阈值（如 90% 模型上限）→ 调小模型总结旧消息为一条摘要，旧消息打 <code>compacted</code> 标记退出后续请求。实现成一个"特殊任务"插进主循环（OpenCode 正是如此）。</li>
        <li><strong>快照</strong>：建一个旁路 git 仓库（工作区指向项目目录，git-dir 放 <code>~/.myagent/snapshots/&lt;session&gt;</code>），每轮工具执行前 <code>git add -A &amp;&amp; git commit</code>；撤销 = <code>git restore</code> 到指定快照。</li>
        <li><strong>fork / resume</strong>：有了消息模型这两个几乎免费——fork = 复制消息到某个分界点；resume = 读历史继续循环。</li>
        <li><strong>防护</strong>：步数上限（agent.steps）+ doom loop 检测（同参数工具调用连续 3 次）。</li>
      </ul>
      <div class="map">对应 OpenCode：<span class="mono">session/compaction.ts</span> · <span class="mono">snapshot/index.ts</span> · <span class="mono">session/processor.ts (DOOM_LOOP_THRESHOLD)</span></div>
    </div>
  </div>

  <!-- L6 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L6</span><h4>多 Provider 抽象 — 一套循环接所有模型</h4><span class="est">≈ 2–3 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>Anthropic / OpenAI / Google / OpenRouter / 本地 Ollama… 用户一条命令切换，含流式输出。</p></div>
      <ul>
        <li><strong>捷径（推荐）</strong>：直接用 <strong>Vercel AI SDK</strong>（<code>ai</code> 包）——OpenCode 的默认路线。
        <code>streamText({ model: anthropic("claude-…"), tools, messages })</code> 就拿到统一的 fullStream；</li>
        <li><strong>模型目录</strong>：接入 <strong>models.dev</strong>（OpenCode 参与维护的开源模型数据库），一个 JSON 拿到所有 provider/model 的定价与能力，本地缓存定期刷新；</li>
        <li><strong>认证</strong>：API key（环境变量/配置文件）起步，OAuth 后补（可参考 OpenCode 的 <code>auth.ts</code> + <code>core/oauth</code>，支持 Copilot/Anthropic 订阅登录）；</li>
        <li><strong>统一事件模型</strong>：把各家流归一成 <code>text-delta / reasoning-delta / tool-call / tool-result / finish</code> 几种事件——上层永远不碰 provider 差异。这是 OpenCode <code>LLMEvent</code> 的思想，也是它能在 AI SDK 与自研运行时之间平滑切换的原因。</li>
      </ul>
      <div class="map">对应 OpenCode：<span class="mono">provider/provider.ts (2000 行目录装配)</span> · <span class="mono">provider/transform.ts</span> · <span class="mono">packages/llm (LLMEvent + 协议适配)</span> · <span class="mono">core/models-dev.ts</span></div>
    </div>
  </div>

  <!-- L7 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L7</span><h4>服务化 — HTTP API + 事件流 + SDK</h4><span class="est">≈ 3–5 天</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>把运行时变成服务器：任何界面、任何程序都能驱动它、观察它。这是 OpenCode 架构的灵魂层。</p></div>
      <ul>
        <li><strong>API 面</strong>（照抄 protocol 包的分组即可）：<code>session.create/list/prompt/abort</code>、<code>message.list</code>、<code>event.subscribe</code>、<code>permission.reply</code>、<code>config</code>、<code>model.list</code>…</li>
        <li><strong>技术选型</strong>：轻量路线用 Hono + 手写 OpenAPI；OpenCode 路线用 Effect HttpApi（定义即文档）。起步阶段 <strong>Hono + zod 校验</strong> 完全够用；</li>
        <li><strong>事件流</strong>：L2 埋下的 <code>onChange</code> 升级为 SSE 端点 <code>/event</code>；所有状态变更（消息更新/工具进度/权限请求/会话状态）都走这一条流；</li>
        <li><strong>SDK</strong>：从 OpenAPI 生成客户端类型（openapi-typescript / hey-api），前端从此不再手写 fetch；</li>
        <li><strong>无头模式</strong>：<code>myagent run "…"</code> = 启动服务器 + 发一条 prompt + 打印事件 + 退出。CI 场景立即可用。</li>
      </ul>
      <div class="map">对应 OpenCode：<span class="mono">protocol/src/groups/*.ts</span> · <span class="mono">server/src/handlers/*.ts</span> · <span class="mono">client/ (生成 SDK)</span> · <span class="mono">cli/cmd/serve.ts + run.ts</span></div>
    </div>
  </div>

  <!-- L8 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L8</span><h4>UI 层 — 终端 TUI 与 Web 界面</h4><span class="est">≈ 1–2 周</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>人机界面。因为 L7，这里<strong>只消费 SDK 与事件流</strong>，不碰任何智能体逻辑。</p></div>
      <ul>
        <li><strong>TUI 选型</strong>：ink（React 式）最快上手；OpenCode 自研了 <strong>opentui</strong>（终端渲染引擎）+ SolidJS 以获得极致性能与组件复用——起步不必如此；</li>
        <li><strong>核心界面三件套</strong>：消息流渲染（markdown/代码高亮/工具卡片）、输入框（多行 + 文件@引用）、权限确认弹层；</li>
        <li><strong>渲染心法</strong>：UI = <code>fold(事件流)</code>。所有界面状态由 SSE 事件累积而来，断线重连后拉一次全量再续流；</li>
        <li><strong>Web 版</strong>：同一套事件流 + 任意前端框架，两天就能出一个聊天式界面（OpenCode 的 app 包即 SolidJS 实现）。</li>
      </ul>
      <div class="map">对应 OpenCode：<span class="mono">tui/src/app.tsx</span> · <span class="mono">tui/src/context/sdk.tsx</span> · <span class="mono">app/src/ (Web)</span> · <span class="mono">desktop/ (桌面壳)</span></div>
    </div>
  </div>

  <!-- L9 -->
  <div class="layer">
    <div class="layer-head"><span class="layer-badge">L9</span><h4>生态扩展 — 子代理、插件、MCP、技能</h4><span class="est">≈ 持续</span></div>
    <div class="layer-body">
      <div class="goal"><span class="g-label">目标</span><p>让系统从"你的工具"变成"大家的平台"。四件事互相独立，可任意顺序做。</p></div>
      <ul>
        <li><strong>子代理</strong>：实现 <code>task</code> 工具 = 创建子会话 + 递归主循环 + 结果回传。给不同 agent 配不同提示词/模型/权限（搜索代理、计划代理…）；</li>
        <li><strong>插件</strong>：定义钩子接口（<code>tool.execute.before/after</code>、<code>chat.params</code>、自定义工具注册），从配置声明加载本地/远程模块；</li>
        <li><strong>MCP</strong>：引入 <code>@modelcontextprotocol/sdk</code>，把外部 MCP server 的 tools 包进你的注册表——你立刻获得整个 MCP 生态的工具；</li>
        <li><strong>技能/斜杠命令</strong>：markdown 即能力。frontmatter 描述 + 正文按需注入；斜杠命令 = 消息模板。</li>
      </ul>
      <div class="map">对应 OpenCode：<span class="mono">tool/task.ts</span> · <span class="mono">plugin/ + packages/plugin</span> · <span class="mono">mcp/index.ts</span> · <span class="mono">skill/ + command/</span></div>
    </div>
  </div>

  <div class="callout">
    <div class="co-title">节奏建议</div>
    <p><strong>第 1 周</strong>做完 L0–L2，你就有一个能持久化、能干活的内核；<strong>第 2 周</strong> L3–L4 补上灵魂与安全；
    <strong>第 3–4 周</strong> L5–L6 达到"日常可用"；之后 L7–L9 按需展开。每一层结束时都拿真实任务压测
    （"给这个仓库加个功能"），你会精确体会到下一层为什么必须存在——这正是 OpenCode 演化出每个子系统的原因。</p>
  </div>
</section>

<!-- ================= 07 ================= -->
<section id="s7">
  <div class="sec-head"><span class="sec-num">07</span><h2>附录：OpenCode 源码阅读顺序</h2></div>
  <p>如果你想反过来深入本仓库，按这条链读，每一步都建立在上一部的理解上：</p>
  <div class="tbl-wrap">
    <table>
      <thead><tr><th>#</th><th>文件</th><th>看什么</th></tr></thead>
      <tbody>
        <tr><td>1</td><td class="path">packages/opencode/src/index.ts</td><td>CLI 入口，所有命令一览</td></tr>
        <tr><td>2</td><td class="path">src/cli/cmd/run.ts</td><td>非交互模式：一次 prompt 的完整客户端视角</td></tr>
        <tr><td>3</td><td class="path">src/session/prompt.ts → runLoop</td><td>主循环本体（全文档 M1 的出处）</td></tr>
        <tr><td>4</td><td class="path">src/session/processor.ts</td><td>LLM 事件 → 消息部件的翻译器，含 doom loop</td></tr>
        <tr><td>5</td><td class="path">src/session/llm.ts</td><td>AI SDK 接线、工具修复、运行时选择</td></tr>
        <tr><td>6</td><td class="path">src/tool/tool.ts → read.ts / edit.ts / shell.ts</td><td>工具框架与三个最有代表性的工具</td></tr>
        <tr><td>7</td><td class="path">src/permission/index.ts</td><td>Deferred 阻塞式权限确认</td></tr>
        <tr><td>8</td><td class="path">src/session/system.ts + prompt/*.txt</td><td>系统提示的组装与模型家族模板</td></tr>
        <tr><td>9</td><td class="path">src/snapshot/index.ts + session/compaction.ts</td><td>git 快照与上下文压缩</td></tr>
        <tr><td>10</td><td class="path">packages/protocol + server + client</td><td>协议三件套：定义 → 实现 → 生成</td></tr>
        <tr><td>11</td><td class="path">CONTEXT.md + AGENTS.md</td><td>官方架构词汇表与贡献规范（V2 设计思想）</td></tr>
      </tbody>
    </table>
  </div>
  <p>另有两个阅读提示：仓库根的 <code>CONTEXT.md</code> 是官方"领域语言手册"，定义了 System Context / Session Drain / Provider Turn 等术语，
  读 V2 代码前值得先扫一遍；<code>AGENTS.md</code> 则记录了 Effect 风格约定与依赖方向红线。</p>
</section>

</main>

<footer>
  <div class="wrap">
    基于 <span class="mono">anomalyco/opencode</span> 仓库（dev 分支，2026-08）源码剖析编写 · 行号与细节可能随版本漂移，阅读时以代码为准
  </div>
</footer>

</body>
</html>
