# elle 编译到 wasm32-unknown-unknown：移植状态

分支 `wasm`。目标是让 **elle 的 lib** 在 `wasm32-unknown-unknown` 上编过，
作为浏览器里 cordis 插件系统 demo 的底座（demo 只演示插件系统 / policy 热加载 /
拒绝即提问，不搬完整 agent，不接真 LLM）。

**当前状态：M2 完成 —— Elle 代码真的在 wasm 里跑出正确答案了**
（node 里 8 个表达式全过：算术、闭包、`let`、字符串、列表）。
wasm32 上 0 编译错误 0 警告，原生构建全程未受影响（`cargo test --lib` 2275 passed）。

## 怎么验证

```sh
cargo check --lib --no-default-features --target wasm32-unknown-unknown
cargo check --lib          # 必须始终 exit=0，这是硬约束
```

**但从 M2 起，上面两条已经不够了。** wasm32 上最贵的缺陷全都编得过、只在调用时
炸（见 M2 一节的表）。真正的验收是在顶层仓库跑：

```sh
cd ../cordis-wasm && wasm-pack build --target nodejs --dev && node test-node.cjs
```

改了 elle 里任何 wasm 相关的东西，都要跑这一条，`cargo check` 说不了话。

第二条是红线：这个移植的全部价值在于不给原生路径添乱。每次改完两条都要跑。

`--no-default-features` 是必要的：`jit`/`ffi`/`repl`/`plugin` 都进不了 wasm。

M1 收尾时另外跑过、都是 exit=0 的面（改动碰了 feature gate，所以值得留着复跑）：

```sh
cargo check --lib --no-default-features                 # 曾有 4 个错，见下
cargo check --lib --no-default-features --features ffi
cargo check --lib --no-default-features --features plugin
cargo check --tests --examples
```

## 已完成

| commit | 内容 | 错误数 |
|---|---|---|
| `46ea4bf2` | io 整块切除、82 个 primitive stub、`platform_tables()` | 518 → 26 |
| `3f6254fc` | chan 保类型砍实现；clock/cpu、ffi/malloc、ffi/free、import 各自处理 | 26 → 5 |
| `7a014b92` | 三个 IoRequest 使用点：`stream`/`kwarg` 整块 cfg，`make_poll_fd` 给 wasm 对偶 | 5 → 2 |
| `c75f6104` | `port.rs` 的 `OwnedFd` 替身（不可居留 enum） | 2 → 1 |
| `22007e24` | `ffi/registry.rs` 的 dlopen 按 `libloading` feature gate | 1 → **0** |

具体改了什么见那五个 commit message，写得比这里详细。

## M1 的三个错误各自怎么解的

**前三个（`plugin_api` / `kwarg` / `stream`）走的是 B**，且 B 的前提复查后成立 ——
见下面「可复用的结论」第 7 条，那不只是"没影响"，是**机制上拦住了**。

一个比原方案更好的细节：`stream::PRIMITIVES` 不要搬去 `platform_tables()`，
而是**就地**在 `ALL_TABLES` 里加 `#[cfg(not(target_arch = "wasm32"))]`
（`introspection::MLIR_PRIMITIVES` 早有这个先例，所以第 25 行"static 数组不能有
条件项"那句注释其实不准）。搬走会把它之后所有 primitive 的 id 在**原生**上也
重新编号，为一个 wasm-only 的改动作废原生的 stdlib cache；就地 cfg 则原生 id
一个都不动。

`kwarg` 没有自己的 PRIMITIVES 表，所以砍掉它不欠任何名字，也**不要**加进
`gen-wasm-stubs.sh` 的 MODULES。加进去的只有 `stream`（6 个名字 + 3 个别名，
82 → 91）。

**第四个（`port.rs` 的 `OwnedFd`）用了替身，但替身是空 enum 而不是 struct。**
`mod port` 不能跟着 `mod io` 走 —— `value/display` 要渲染 port，`value/send` 要
重建三个 stdio 种类。而 wasm 上根本进不来一个 fd：所有取 fd 的入口和读 fd 的
访问器都只被 `io`/`net`/`unix`/`ports` 调用，而存活的 stdio 构造器一律
`fd: None`。所以用不可居留类型，让"这里不可能有 fd"变成编译器检查的事实，而不是
一句注释。`port.rs` 内部从不对 fd 调方法（只在 `Option` 里搬进搬出），所以替身
不需要配任何 impl。

**第五个（libloading）根本不是 wasm 问题。** 见下面第 8 条。

## 可复用的结论

这些是这轮验证出来的，不要重新调查：

1. **`vm`/`compiler`/`reader`/`value` 对 `crate::io` 零耦合**，核心求值器是纯计算。
   fiber 是 bytecode 级的（frame 交换，不是栈切换），wasm 上可行。
2. **fiberheap 不需要 MMU。** 页可以来自
   `alloc_zeroed(Layout::from_size_align(len, len))` —— `Layout` 的对齐直接给出
   mmap 路径要靠 over-allocate + trim 才能得到的东西。失去的只有 guard page
   诊断（`--trace=guardfree`）和 atexit 直方图，都不是功能。
3. **`chan`、`ffi`、`plugin_api` 三个模块不能整块 cfg 掉。** 它们的纯数据部分早已
   渗进 `value/heap`、`value/display`、`value/send`、`vm/core`：
   - chan → `SendableValue`、`WakeList`（`ThreadHandle` 和 `SendValue` 的变体要用）
   - ffi / plugin_api → 试过整块砍，错误从 14 涨到 20，已撤回
   能砍的只有实现，永远不是模块本身。
4. **`io::request` 不是纯数据层。** `request.rs` 自己对 io 零依赖，但四个子模块
   不是：`SocketOptions::apply` 调 `setsockopt`，`spawn.rs` 调
   `crate::io::completion_heap_ptr`，`buffer.rs` 调 `crate::io::io_error`。
5. **Elle 里未绑定的名字是编译错误**，所以被砍掉的 primitive 必须留下名字。
   这是 `stub_wasm.rs` 存在的唯一理由，也是 `stdlib.lisp` 3710 行能一行不改的原因
   （`ev/spawn`、`chan/select`、`subprocess/system` 在 stdlib 里只是 `defn`
   定义，加载时不执行）。
6. **primitive id 是它在 `ALL_TABLES` + `ffi_tables()` + `platform_tables()`
   枚举中的索引。** 六个消费点必须按同一顺序 chain。改表结构前先想清楚这一点。
7. **"wasm 与原生 primitive id 不同"是安全的，而且是机制保证的，不是靠约定。**
   这条查过了，不用再查：
   - id 由 `PRIM_REGISTRY`（一个 `LazyLock`）在运行时按表序生成。没有 build
     script，没有编译期烘焙。
   - 仓库里**根本不存在** `.elleb` —— 这个名字只在本文档里出现过一次。没有那种
     跨 target 共享的预编译产物格式。
   - 全仓库唯一把 prim_id 落盘的地方是 `compiler/stdlib_cache.rs`。它的 cache key
     同时混入 `build_identity()`（可执行文件的长度 + mtime）和
     `hash_prim_table_identity()`（每个 def 的 name + aliases，按序）。所以就算
     两个 target 共用一个 cache 目录，表不同 → key 不同 → 读不到对方的。
   - wasm 上还有第三层：`build_identity()` 走 `std::env::current_exe()`，
     wasm32-unknown-unknown 答不出来 → `cache_key()` 返回 `None` →
     `try_load`/`try_store` 都直接 no-op。cache 在 wasm 上结构性关闭。
   - `value/send/ser.rs` 里那句"prim_id stable across threads/processes"指的是
     **同一个 build** 的进程；send 是线程间的，而 wasm 上没有线程。
8. **`ffi/registry.rs` 依赖 `libloading`，而那是 optional dep（只有 `ffi` 和
   `plugin` feature 拉它）。** 这是个与 target 无关的预存 bug：原生
   `cargo check --lib --no-default-features` 当时报 4 个同样的错。所以**该按
   feature gate，不是按 target gate** —— 修完顺手把原生那条也修好了。
   optional dep 会得到一个同名的隐式 feature，所以 `#[cfg(feature = "libloading")]`
   就是"这个 crate 在不在"的最精确写法。
   `load`/`load_self` 本来就有 `#[cfg(unix)]` 的失败兜底分支，真正的条件是
   `all(unix, feature = "libloading")`。
   顺带一个语言坑：**`#[cfg(...)]` 里不能用 `macro_rules!` 展开的条件**（属性不做
   宏展开），所以这种复合条件只能每处抄一遍。
9. **wasm32 上「编过」和「链接过」都不说明能跑，而且这道沟特别宽。**
   缺失的能力在这个 target 上大多不是缺符号，而是**能编译的 panic 实现**：
   `Instant::now()` 编得过、一调就 `time not implemented on this platform`；
   `eprintln!` 编得过、输出被丢弃。所以每一个用到 OS 能力的地方都得问"调用时
   会怎样"，`cargo check` 和 `cargo build` 都答不了。M2 找到的四个缺陷全属此类。
   推论：**移植的验收必须是执行，不能是编译**。
10. **stub 一个名字，和让那个名字能用，是两件事 —— 取决于谁在什么时候调它。**
    "绑定到报 `:unsupported` 的 primitive"对*用户程序可能调用*的名字是对的
    （第 5 条），但对 **stdlib 自己在 `init_stdlib` 期间就要调用**的名字是致命的：
    stdlib.lisp 顶层的 `(def *stdout* (parameter (port/stdout)))` 让三个被 stub 的
    port 构造器直接搞掉整个 stdlib 加载 —— 后果不是 `println` 不工作，而是
    **`+` 和 `let` 都不存在**。切模块时要单独检查一遍：被砍掉的名字里，有哪些是
    stdlib 在**加载期**（而非函数体内）调用的。判据是在 stdlib.lisp 里 grep 那些
    名字，看缩进和是否在 `defn` 体内。

## 两个方法论教训

这轮同一个错误犯了两次，都是同一个形状：

> **「某个模块是纯数据 / 可以整块砍掉」这种判断，必须查到叶子文件。
> 顶层文件的 import 列表说明不了模块树的依赖。**

两次分别是 `mod ffi`（14 → 20）和 `io::request`（5 → 29）。正确的做法是动手前
先 grep 该模块**所有** `.rs` 叶子对外部的引用，而不是只看 `mod.rs` 或同名文件。

另一个小坑：从 `primitive!` 块提取名字时，不能裸 grep `"x" =>` —— `io.rs` 里有
`"read" => libc::POLLIN` 这类 match 分支会假阳性。`gen-wasm-stubs.sh` 限定在
`primitive!` 块内提取，并且提取到少于 40 个名字就硬失败。

## M2：`eval` 在 wasm 里跑出正确答案

新增 `cordis-wasm` crate（顶层仓库，不动 `cordis-pi`），导出
`eval_source(src) -> String`，`node cordis-wasm/test-node.cjs` 是验收。

**M2 的全部价值在于"编过"和"跑对"之间的那道沟。** M1 结束时 wasm32 是 0 编译
错误，甚至 `cargo build --lib --target wasm32-unknown-unknown` 也能链接通过
（10.6s）—— 而下面四个缺陷一个都没被这些命令发现，因为它们**全都是编过、
调用时才炸**：

| 缺陷 | 症状 | 解 |
|---|---|---|
| `Instant::now()` 在 wasm32 上 panic | `time not implemented on this platform` | `trace::stamp()`：原生 `Instant`，wasm `()` |
| stdlib 顶层调 `(port/stdout)`，撞上 stub | `init_stdlib` 直接失败 → **没有 stdlib，没有 `+`，没有 `let`** | `primitives/stdio_wasm.rs` 给三个真实现 |
| `eval_all` 把代码包进 `ev/run` | scheduler 第一步 `(io/backend :async)` 要 epoll/kqueue，连 `(list 1 2 3)` 都跑不了 | 改用 `eval_file`（裸 `execute`） |
| wasm32 丢弃 stderr | panic 只显示哑 `unreachable`，看不到任何信息 | `console_error_panic_hook` |

第二条最值得记：**"名字绑上了"对 stdlib 在启动路径上要调的 primitive 是不够的。**
M1 的 stub 策略（每个被砍的名字都绑到一个报 `:unsupported` 的 primitive）对
"程序可能调、调了就该收到错误"完全正确，但 `port/stdin`/`port/stdout`/`port/stderr`
是 stdlib 自己在 `init_stdlib` 期间就要调的，一个会失败的绑定等于没有绑定。
而这三个**根本不需要 OS** —— 它们只是造一个 `fd: None` 的 `Port` 值，这正是
M1 特意让 `mod port` 在 wasm 上活着的原因；它们被 stub 纯粹是因为跟一堆真要
建 `IoRequest` 的 primitive 同处一个模块（`ports`）。

为了让两张表不重叠，`gen-wasm-stubs.sh` 加了 `PROVIDED` 列表跳过这三个名字
（91 → 88 个 stub），**并且在 PROVIDED 里的名字在被扫模块中不存在时硬失败** ——
否则将来一次改名会静默地把 stub 装回去，而爆点（stdlib 加载失败）离改名处极远。

第三条同样是个入口选择问题而不是移植问题，`eval_all` / `eval_file` 的区别见
`docs/elle-lisp-knowledge.md` 第 70 条。

**尚未处理的运行时 panic 隐患**（都不在 `(+ 1 2)` 路径上，所以 M2 没碰）：
`primitives/time.rs` 的 `process_epoch()`（`OnceLock<Instant>` + `Instant::now`，
供 `clock/monotonic`）和 `prim_clock_realtime` 的 `SystemTime::now()`。
形状和上表第一条完全一样，**会在第一次调用时炸，不会在编译期出现**。
（`clock/cpu` 在 M1 已处理：wasm 下返回 `unsupported` 错误，不拿墙钟冒充。）

## 已知问题

**`port/*` 的现状在 M2 变了，下面这段是订正后的。** 原先这里写的是
"`port/stdout` 和 `port/stderr` 跟着被 stub 了，所以 `println` 是
`:unsupported`" —— 那个判断把后果说小了：stdlib.lisp 在**顶层**就调这三个
构造器（`(def *stdout* (parameter (port/stdout)))`），所以 stub 不是让
`println` 失败，而是让 `init_stdlib` 整个失败，wasm 上连 `(+ 1 2)` 都没有。
详见下面 M2 一节。

现在这三个是真实现（`primitives/stdio_wasm.rs`），`*stdin*`/`*stdout*`/`*stderr*`
是真 port 值。**仍然缺的是运输**：`port/write`/`port/read` 还是 stub，所以
`println` 依旧不出字。**这部分不是待修的 bug，是设计的一部分**：demo 的输出必须由
embedder（`cordis-wasm` 那一层的 mock host）注册 JS console 回调来提供。
M4 之前必须补上，否则看不到任何输出。

警告已清零（`339bc7bd`）。**wasm 与全部原生 feature 组合都是 0 警告 0 错误。**

那 81 个警告正好沿三条**互不相干**的轴分解，这是清理能落在对处的原因：

| 数量 | 位置 | 轴 |
|---|---|---|
| 54 | `plugin_api/` 子树 | `feature = "plugin"` |
| 7 | `ffi/registry.rs` | `feature = "libloading"` |
| 20 | chan / port / heap / vm / freelog | `target_arch = "wasm32"` |

**前两组根本不是 wasm 问题** —— 原生 `--no-default-features` 同样报 61 个，而
`--features plugin` 单独开就 0 个。按 target 去 gate 它们会"恰好也对"，但那是巧合。

- lint 等级**沿模块树下传**，所以 54 个用 `pub mod plugin_api;` 上一个
  `#[cfg_attr(not(feature = "plugin"), allow(dead_code))]` 就全收了。
- 20 个里有 6 个是真垃圾（`chan.rs` 里只被切除的 primitive 体用到的 import），
  那 6 个是 **cfg 掉，不是 allow 掉**。
- 剩 14 个是本移植通用的"保数据、砍实现"，allow 才是诚实的答案。但**先查到真正的
  调用者再标**：`exit_trapped` 的读者在 `primitives::subprocess`、
  `ThreadHandle::new` 的在 `primitives::concurrency`、`freed_site` 的是
  `segv_handler`（需要 MMU）。
- `port` 和 `chan` 整个模块在 wasm 上只剩数据，用模块级 allow；另外 4 个散落在
  仍然活着的大模块里，**逐项标注**，这样那些模块将来真出现死代码还会报。

唯一剩下的警告：`cargo check --tests --examples` 报 `io::pending::get` never
used。**预存且与本移植无关**（`io/` 全程没碰，且只在 test build 里出现）。

## 后续里程碑

M1、M2（本文档）之后：

- ~~**M2** — node 里 `eval("(+ 1 2)")` 得到 3。~~ **已完成**，见上面 M2 一节。
  实际只需要一个导出函数（`eval_source`）而不是预想的三个：`init()` 和
  `store()` 都属于"跨调用持久状态"，而那是 M3 的事 —— 见下。
  规模也比预估小得多（`src/lib.rs` 69 行），因为难点全在 elle 侧的四个运行时
  缺陷上，不在 binding 上。
- **M3** — 内存 import resolver 加载 `loader.lisp`，外加**跨调用持久 VM**。
  **这是剩下最大的风险**：浏览器没有文件系统，而 `cordis-elle` 用
  `import-file`/`include` 组织 13 个文件。方案是 `include_str!` 打包 + 给 elle 的
  路径解析注入内存 resolver。

  M2 查实的两件事改变了这一步的形状：

  1. **好消息** —— elle 自己的 stdlib 是 `include_str!("../stdlib.lisp")`
     （`primitives/module_init.rs`），本来就不碰文件系统。所以需要内存 resolver
     的只有 cordis-elle 那 13 个文件，不含 elle 自身。
  2. **新的必做项** —— 持久 VM 不只是优化。M2 每次调用都重建引擎，因此每次都要
     重新编译一遍整个 stdlib：node 里 8 个表达式 24 秒（dev 构建，未优化）。
     加载 cordis-elle 之后按 per-call 重建根本不可行。
     难点是 `VM` 持有指向它自己的 `SymbolTable` 和 `CompileCtx` 的裸指针
     （`vm.set_compile_ctx(&mut cctx as *mut _)`），所以这三件套是自引用的，
     不能直接塞进 `thread_local`。
- **M4** — `demo-harness.lisp`（改自 `cordis-elle/examples/agent-harness.lisp`，
  161 行）输出与原生一致。**M4 之前不写任何 UI 代码。**
- **M5** — 三面板 UI。

`cordis-elle` 是 1670 行纯 Elle Lisp、0 个 `.rs`，只用了一个 io primitive
（`file/write`），coeffect / 生命周期 / SUSPENDED 机制全在里面。它**预期零改动**。
`cordis-pi` 那 10615 行 Rust 原型完全不需要。

wasm 产物体积预估 3–8MB（全量 reader + compiler + stdlib），原型可接受。
