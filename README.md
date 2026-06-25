MiniLisp-RS
=============

一个用 Rust 编写的简易 Lisp 解释器雏形。

lv0 词法分析器
交互示例：

```
>>> (+ 1 2)
(LEFT_PAREN)
(IDENTIFIER +)
(NUMERIC_LITERAL 1.000000)
(NUMERIC_LITERAL 2.000000)
(RIGHT_PAREN)
>>>
```

当前文件结构:


在 Windows 上结束输入以退出：按 Ctrl+Z 然后回车。

# MiniLisp-RS
A lightweight Mini Lisp interpreter built in Rust for learning compiler and interpreter fundamentals.


