# elle 编译到 wasm32-unknown-unknown：移植状态

分支 `wasm`。目标是让 **elle 的 lib** 在 `wasm32-unknown-unknown` 上编过，
作为浏览器里 cordis 插件系统 demo 的底座（demo 只演示插件系统 / policy 热加载 /
拒绝即提问，不搬完整 agent，不接真 LLM）。

**当前状态：5 个编译错误。原生构建全程未受影响。**

## 怎么验证

```sh
cargo check --lib --no-default-features --target wasm32-unknown-unknown
cargo check --lib          # 必须始终 exit=0，这是硬约束
```

第二条是红线：这个移植的全部价值在于不给原生路径添乱。每次改完两条都要跑。

`--no-default-features` 是必要的：`jit`/`ffi`/`repl`/`plugin` 都进不了 wasm。

## 已完成

| commit | 内容 | 错误数 |
|---|---|---|
| `46ea4bf2` | io 整块切除、82 个 primitive stub、`platform_tables()` | 518 → 26 |
| `3f6254fc` | chan 保类型砍实现；clock/cpu、ffi/malloc、ffi/free、import 各自处理 | 26 → 5 |

具体改了什么见那两个 commit message，写得比这里详细。

## 剩下的 5 个

```
plugin_api.rs:13       use crate::io::request::IoRequest
primitives/kwarg.rs:6  use crate::io::request::SocketOptions
primitives/stream.rs:7 use crate::io::request::{IoRequest, PortOp}
port.rs:9              use std::os::unix::io::OwnedFd
ffi/registry.rs:48     libloading::Library
```

前三个是同一件事，有三条路：

| | 做法 | 代价 | 评价 |
|---|---|---|---|
| A | 深挖 `io/request` 的子模块，把 `setsockopt`/`spawn`/`buffer` 里的执行层调用逐个 cfg | 一动就是 29 个错误起步，且不知道还有几层 | **已试过，撤回了** |
| **B** | `stream`/`kwarg` 整块 cfg，primitive 名字补进 `stub_wasm` | wasm 下 primitive id 与原生不同 | **推荐** |
| C | 各处本地定义替身类型 | `SocketOptions` 是结构体字段类型，会在两个 target 上分叉成两套 | 不建议 |

推荐 B 的理由：`platform_tables()` 这套机制已经建好，把模块整组移出 `ALL_TABLES`
正是它的用途；`tools/gen-wasm-stubs.sh` 可以重跑，补 `stream` 的名字是改一行参数。
它完全不动 `io/` 内部，对上游同步负担最轻。

**B 的前提**：wasm 与原生的 primitive id 不同必须是可接受的。id 是每次构建按表
枚举顺序生成的，只要不跨 target 共享预编译 `.elleb` 就没有影响 —— 浏览器 demo
是 eval 源码，不涉及。动手前确认这个前提仍然成立。

后两个错误与上面的选择无关，是两件独立的活：

- `port.rs` 的 `OwnedFd`，16 处，File port 的 fd 所有权。要么给个替身 struct，
  要么把 File port 整块 cfg。
- `ffi/registry.rs` 的 libloading，6 处，集中在 `LoadedLib`。注意不能把
  `mod ffi` 整个 cfg 掉（见下）。

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
