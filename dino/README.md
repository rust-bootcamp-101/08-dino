# 构建 dino 命令行

- 构建 dino init: init project (git init), create config file, and main.ts
- 构建 dino build: 遍历目录下所有 ts/js/json 文件，生成一个 hash 作为 bundle 的文件名写入 .build 目录下
- 构建 dino run
    - rquickjs: js runtime
    - 可以在本地运行 bundle 好的代码
