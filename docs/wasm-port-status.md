# elle 编译到 wasm32-unknown-unknown：移植状态

分支 `wasm`。目标是让 **elle 的 lib** 在 `wasm32-unknown-unknown` 上编过，
作为浏览器里 cordis 插件系统 demo 的底座（demo 只演示插件系统 / policy 热加载 /
拒绝即提问，不搬完整 agent，不接真 LLM）。

**当前状态：M1 完成 —— wasm32 上 0 个编译错误。原生构建全程未受影响。**

## 怎么验证

```sh
cargo check --lib --no-default-features --target wasm32-unknown-unknown
cargo check --lib          # 必须始终 exit=0，这是硬约束
```

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

## 两个方法论教训

这轮同一个错误犯了两次，都是同一个形状：

> **「某个模块是纯数据 / 可以整块砍掉」这种判断，必须查到叶子文件。
> 顶层文件的 import 列表说明不了模块树的依赖。**

两次分别是 `mod ffi`（14 → 20）和 `io::request`（5 → 29）。正确的做法是动手前
先 grep 该模块**所有** `.rs` 叶子对外部的引用，而不是只看 `mod.rs` 或同名文件。

另一个小坑：从 `primitive!` 块提取名字时，不能裸 grep `"x" =>` —— `io.rs` 里有
`"read" => libc::POLLIN` 这类 match 分支会假阳性。`gen-wasm-stubs.sh` 限定在
`primitive!` 块内提取，并且提取到少于 40 个名字就硬失败。

## 已知问题

`port/stdout` 和 `port/stderr` 跟着被 stub 了，所以 wasm 下 `println` 是
`:unsupported`。**这不是待修的 bug，是设计的一部分**：demo 的输出必须由
embedder（`cordis-wasm` 那一层的 mock host）注册 JS console 回调来提供。
M4 之前必须补上，否则看不到任何输出。

wasm 上还有 81 个警告（0 错误）。**不用急着清，但要知道它们是什么**：

- 6 个 unused import，全在 `primitives/chan.rs`（`3f6254fc` 保类型砍实现留下的）。
- 其余基本都是 `dead_code`，绝大部分是 `plugin_api/capi.rs` 那套 C ABI 访问器 ——
  没有 `plugin` feature 就没人调它们。它们的存在是有意的（ABI 表形状不能随
  target 变），所以这批警告的正确处理多半是给模块加一个 `#![allow(dead_code)]`
  而不是删代码。
- `ffi/registry.rs` 的 `NativeLib`/`LoadedLib`/`REGISTRY` 等也在里面，同理：
  bookkeeping 保留，只是 wasm 上永远不会有东西进去。

清警告前先确认它不会引入 target 分叉的 `#[cfg]` 噪声 —— 现在这批警告的信息量
（"这些路径在 wasm 上确实死了"）本身是有价值的。

## 后续里程碑

M1（本文档）之后：

- **M2** — node 里 `eval("(+ 1 2)")` 得到 3。需要新建 `cordis-wasm` crate
  （~200 行，wasm-bindgen，三个函数 `init()` / `eval(src)` / `store()`），
  不动 `cordis-pi`。
- **M3** — 内存 import resolver 加载 `loader.lisp`。**这是剩下最大的风险**：
  浏览器没有文件系统，而 `cordis-elle` 用 `import-file`/`include` 组织 13 个文件。
  方案是 `include_str!` 打包 + 给 elle 的路径解析注入内存 resolver。
- **M4** — `demo-harness.lisp`（改自 `cordis-elle/examples/agent-harness.lisp`，
  161 行）输出与原生一致。**M4 之前不写任何 UI 代码。**
- **M5** — 三面板 UI。

`cordis-elle` 是 1670 行纯 Elle Lisp、0 个 `.rs`，只用了一个 io primitive
（`file/write`），coeffect / 生命周期 / SUSPENDED 机制全在里面。它**预期零改动**。
`cordis-pi` 那 10615 行 Rust 原型完全不需要。

wasm 产物体积预估 3–8MB（全量 reader + compiler + stdlib），原型可接受。
