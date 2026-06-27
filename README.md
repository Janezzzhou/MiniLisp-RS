# MiniLisp-RS

一个使用 Rust 编写的轻量级 Mini Lisp 解释器，提供两种使用方式：

- 命令行 REPL / 文件执行
- 基于 `eframe/egui` 的桌面 GUI

## 给使用者

### 目前支持的功能

MiniLisp-RS 目前支持：

- 在终端中交互式执行 Lisp 表达式
- 从 `.scm` 文件运行 Lisp 代码
- 使用 GUI 窗口输入代码并查看输出结果
- 常见的 Lisp 风格语言特性，包括：
  - 算术与比较
  - `define`、`lambda`、`if`、`begin`、`cond`、`let`
  - 列表、序对、引用、准引用
  - 内建过程，如 `map`、`filter`、`reduce`、`car`、`cdr`、`cons`

### 运行 CLI 版本

启动 REPL：

```bash
cargo run
```

运行一个 Scheme/Lisp 文件：

```bash
cargo run -- sort_test1.scm
```

在 Windows 上，可以按 `Ctrl+Z` 再按回车结束终端输入。

### 运行 GUI 版本

启动 GUI：

```bash
cargo run --bin gui
```

构建 release 可执行文件：

```bash
cargo build --release --bin gui
```

生成的可执行文件位置为：

```text
target/release/gui.exe
```

在 Windows 上你可以直接双击 `gui.exe` 打开图形界面。

### GUI 使用说明

在 GUI 中：

- 左侧面板：输入 Lisp 代码
- 右侧面板：查看输出和求值结果
- `Run`：执行当前输入
- `Clear Output`：清空输出面板
- `Reset Env`：重置解释器状态
- `Ctrl+Enter`：快速执行当前输入

输入编辑器还支持：

- 语法高亮
- 括号匹配高亮

### 示例代码

你可以把下面这些示例粘贴到 CLI REPL 或 GUI 输入面板中运行。

基础算术：

```lisp
(+ 1 2 3 4)
(* 6 7)
(/ 7 2)
```

变量与表达式：

```lisp
(define x 10)
(define y 32)
(+ x y)
```

函数定义：

```lisp
(define (square x) (* x x))
(square 12)
```

条件判断：

```lisp
(define n -5)
(if (> n 0) "positive" "not positive")
```

列表：

```lisp
(list 1 2 3 4)
(car '(10 20 30))
(cdr '(10 20 30))
(append '(1 2) '(3 4))
```

Lambda：

```lisp
((lambda (x y) (+ x y)) 3 4)
```

映射与过滤：

```lisp
(map (lambda (x) (* x x)) '(1 2 3 4))
(filter (lambda (x) (> x 2)) '(1 2 3 4))
```

递归风格辅助函数示例：

```lisp
(define (double x) (+ x x))
(map double '(1 2 3 4))
```

## 给开发者

### 常用开发命令

检查整个项目：

```bash
cargo check
```

仅检查 GUI：

```bash
cargo check --bin gui
```

运行测试：

```bash
cargo test
```

构建所有目标：

```bash
cargo build
```

构建 release 可执行文件：

```bash
cargo build --release --bin gui
```

生成的可执行文件位置为：

```text
target/release/gui.exe
```

### 项目结构

解释器核心：

```text
src/lib.rs           对外库入口与 execute_source
src/main.rs          CLI 入口：REPL 与文件执行
src/token.rs         Token 定义
src/tokenizer.rs     词法分析 / 分词
src/parser.rs        从 token 解析为 Lisp 值
src/value.rs         运行时值表示
src/eval_env.rs      求值环境与求值器
src/form.rs          特殊表单
src/builtins.rs      内建过程
src/output.rs        CLI 与 GUI 共用的输出抽象
src/error.rs         错误类型
```

GUI：

```text
src/bin/gui.rs           GUI 入口、布局、交互流程
src/bin/gui/theme.rs     GUI 主题与 egui 样式应用
src/bin/gui/editor.rs    编辑器高亮与装饰逻辑
```

### GUI 代码组织方式

GUI 代码按职责拆分如下：

- `gui.rs`
  - 管理应用状态
  - 连接按钮与面板交互
  - 调用解释器执行代码
- `theme.rs`
  - 保存颜色、间距、尺寸、字体等视觉设置
  - 将主题应用到 egui
- `editor.rs`
  - 负责语法高亮
  - 负责当前行和括号匹配的装饰效果

如果你想：

- 修改颜色、间距、字体大小：编辑 `src/bin/gui/theme.rs`
- 修改编辑器高亮规则：编辑 `src/bin/gui/editor.rs`
- 修改窗口布局或按钮行为：编辑 `src/bin/gui.rs`

### 推荐开发流程

1. 在 `src/lib.rs` 及相关解释器模块中进行核心语言修改。
2. 使用 `cargo test` 验证行为。
3. 如果改动影响 GUI，运行：

```bash
cargo check --bin gui
```

4. 如果你修改了布局或样式，本地运行 GUI：

```bash
cargo run --bin gui
```

### 扩展解释器时的说明

当你要新增一个语言特性时，通常流程是：

1. 如果需要，先在 `tokenizer.rs` 中补充分词逻辑。
2. 在 `parser.rs` 中更新解析逻辑。
3. 在 `form.rs`、`builtins.rs` 或 `eval_env.rs` 中加入求值逻辑。
4. 在现有测试模块中补充测试。
5. 如果这个特性会影响显示输出，同时验证 CLI 和 GUI 的行为。

## License

参见 [LICENSE](LICENSE)。
