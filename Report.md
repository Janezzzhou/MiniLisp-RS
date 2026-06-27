## C++ Rust 风格差异
### 1. 内存管理：从共享所有权到显式建模的所有权图
这个项目里最直观的差异，首先体现在“值”和“环境”究竟怎么活着。C++ 版本里，很多对象天然会落到 std::shared_ptr 上，写法很直接，例如 using ValuePtr = std::shared_ptr<Value>;，环境链也会自然写成 std::shared_ptr<EvalEnv> parent;。这种方式的优势是表达力强，特别适合解释器这类天然图结构的数据模型；但它把很多正确性问题留到了运行期，例如循环引用、悬空共享、以及“这到底是同一个对象，还是一个结构相同的新对象”。
Rust 版本虽然最终也回到了引用计数，但它逼着我先把“共享”和“可变”拆开来想。值层用了 Rc<Value>，环境层用了 Rc<RefCell<EvalEnv>>，对应代码大致是：
pub type ValuePtr = Rc<Value>;
pub type EnvPtr = Rc<RefCell<EvalEnv>>;

pub struct EvalEnv {
    symbols: HashMap<String, ValuePtr>,
    parent: Option<EnvPtr>,
}
这里最大的感受是，Rust 并不是不允许共享，而是要求你把“为什么要共享”“为什么要内部可变”说清楚。Rc 只解决多方持有，RefCell 才解决运行期可变借用；两者分离以后，环境对象的别名关系会比 C++ 的“一个 shared_ptr 包打天下”更清晰。尤其在 Lambda 捕获定义时环境时，Rust 迫使我明确区分“复制环境内容”和“共享环境对象”这两个完全不同的语义，这反而让闭包的实现更扎实。
### 2. 数据表示：从继承层次到代数数据类型
C++ 版本的 Mini-Lisp 往往会把 Value 设计成一个基类，再派生出 NumberValue、PairValue、LambdaValue 等子类。这样的模型符合传统面向对象直觉，但在解释器里，绝大多数操作其实是“对一种封闭值集合做分类讨论”，而不是多态分发。Rust 的 enum 恰好更贴合这一点：
pub enum Value {
    Boolean(bool),
    Numeric(f64),
    String(String),
    Nil,
    Symbol(String),
    Pair(ValuePtr, ValuePtr),
    BuiltinProc(BuiltinFunc),
    LambdaProc { params: Vec<String>, body: Vec<ValuePtr>, env: EnvPtr },
}
一旦数据结构是封闭的，后续的求值、打印、类型判断就都可以写成 match。例如 Display 实现里，Rust 会强制我穷尽所有分支；而在 C++ 里，如果是基类加 dynamic_cast，遗漏一种派生类型通常只能在运行时才暴露。这个差异背后的设计哲学非常鲜明：C++ 更偏向“对象自己知道如何表现”，Rust 更偏向“调用方穷尽地处理所有可能状态”。对解释器这类程序来说，后者其实更自然，因为语义规则本来就是一张穷尽的表。
### 3. 环境与闭包：从可用实现到语义正确实现
迁移过程中最有代表性的一个转折，是 Lambda 的上级环境。最开始无论在 C++ 还是 Rust，都很容易先写出“能跑”的版本：调用 Lambda 时复制当前符号表，把参数塞进去，再解释函数体。这种实现足以通过很多基础测试，但它并不是真正的词法作用域。后来为了完成闭包捕获，我们在 Rust 中把 LambdaProc 改成显式保存定义时环境：
Value::LambdaProc {
    params,
    body,
    env: env.clone(),
}
调用时则不再基于“当前环境”扩展，而是基于“定义时环境”构造子环境：
let local_env = EvalEnv::with_parent(defining_env);
这一点在 C++ 中通常也能做到，但 Rust 让我更强烈地意识到：闭包的正确性不是某个函数体里的小技巧，而是运行时对象图的一部分。尤其在 let 转 lambda、map/filter/reduce 调用用户过程时，这种词法链条一旦建对，后续很多特性会自然成立。可以说，Rust 在这里带来的不是“更短的代码”，而是更强的语义自觉。
### 4. 错误处理与控制流：异常思维和结果类型思维
C++ 实现里，解释器错误常常有两种路子：一种是返回空指针或特殊值，另一种是直接 throw。前者容易让错误传播变得隐式，后者则会把普通控制流和异常控制流混在一起。Rust 的 Result<T, E> 则迫使错误成为函数签名的一部分。我们的很多核心入口都变成了：
pub fn eval(env: &EnvPtr, expr: ValuePtr) -> Result<ValuePtr, LispError>
一开始这会让代码显得更“啰嗦”，因为几乎每一步都要 ?。但随着内建过程、特殊形式、文件模式一起接上，这种统一反而变成了优点。甚至像 (exit) 这样的“非局部控制流”，最后也被建模成了 LispError::Exit(i32)，由主循环集中处理：
match EvalEnv::eval(&env, expr) {
    Ok(value) => println!("{}", value),
    Err(LispError::Exit(code)) => std::process::exit(code),
    Err(e) => eprintln!("Error: {}", e),
}
这和 C++ 里“抛一个专门的退出异常”很像，但 Rust 的好处在于：这个分支是显式暴露在类型系统里的，阅读者不需要猜测某个函数会不会突然终止整个解释器。
### 5. 编译期约束与实现心智：Rust 更慢，但也更诚实
从体验上说，Rust 实现并不总是“更轻松”。很多在 C++ 里一把梭的写法，到了 Rust 会被借用检查器、所有权规则、函数指针比较警告、Rc<RefCell<_>> 的层层包装打断。例如我们曾经为 BuiltinFunc 是否要带环境参数、EvalEnv::new() 是否还返回 Self、Lambda 是否捕获定义时环境，反复调整过接口。这些在 C++ 里往往可以先写出来再说，之后靠测试修补；而 Rust 更像是在实现过程中不断追问：这个对象到底归谁管，这次可变借用和上一次共享借用会不会冲突，这个控制流是不是应该写进类型里。
但正因为这种“慢”，Rust 也显得更诚实。很多设计问题在 C++ 里会被隐藏成“以后再说的工程细节”，到了 Rust 里则无法回避，必须当场回答。回头看，这种压力其实帮助我把解释器的语义边界想得更清楚了：哪些值是结构相等，哪些是对象恒等；哪些环境是复制，哪些是共享；哪些错误只是普通运行时错误，哪些其实是解释流程控制。对一个教学性质的 Mini-Lisp 来说，这种收获甚至比“最终代码能运行”更重要。
整体而言，这次迁移给我的最大感想是：C++ 更像是在给程序员充分的表达自由，Rust 则是在不断要求程序员把隐含假设外显出来。前者更灵活，后者更约束；前者更像经验驱动，后者更像模型驱动。对于解释器这种既有复杂对象关系、又高度依赖语义一致性的系统程序，Rust 并没有神奇地减少复杂度，但它把复杂度从运行期搬到了编译期，也把很多“可以先凑合”的设计变成了“必须说清楚”的设计。这种变化，正是这次项目中最值得记录的实现差异。