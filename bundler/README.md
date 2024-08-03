crate功能：将TypeScript代码bundle成可供JS Runtime运行的JavaScript(IIFE格式)代码，webpack和swc已经具备了这样的功能，我们可以直接使用它们的能力

[swc](https://swc.rs) 太复杂，学习成本太高

[dune](https://github.com/aalykiot/dune) 自成体系，代码量适中，只用bundle的功能，需要裁剪代码

需要拆解的项目是 [dune](https://github.com/aalykiot/dune), 抽取出JavaScript/TypeScript的运行时
查阅代码，抽取 [bundle.rs](https://github.com/aalykiot/dune/blob/main/src/tools/bundle.rs) 和它相关的依赖即可
