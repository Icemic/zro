# 泽洛 - 神速咏唱 (Quick Spell for Stable Diffusion Prompt)

## 简介 (Introduction)

泽洛 - 神速咏唱 是一款跨平台应用程序（Windows、macOS、Linux、Web），用于快速查找用于 Stable Diffusion 模型的提示词。

Zro - Quick Spell is a cross-platform application (Windows, macOS, Linux, Web) for quickly finding prompts for Stable Diffusion models.

![screenshot](https://github.com/user-attachments/assets/b2f76b48-4b18-41fd-b076-ed43f442f845)

## 使用 (Usage)

1. 直接下载 [Release](https://github.com/Icemic/zro/releases) 版本。该版本已内置提示词数据。Download directly from [Release](https://github.com/Icemic/zro/releases). This version comes with built-in prompt data.
    

2. 自行编译使用：（需要在根目录放置 data.csv 文件作为提示词数据源）。Build from source: (requires placing a data.csv file in the root directory as the prompt data source)

```bash
cargo build --release
```

## 使用外部提示词数据 (Using External Prompt Data)

在程序同目录放置 UTF-8 编码的 data.csv 文件作为提示词数据源，程序会自动读取。提示词数据格式如下：

Place a UTF-8 encoded data.csv file in the same directory as the program to serve as the prompt data source. The program will read it automatically. The prompt data format is as follows:

```csv
tag,tag_cn
1boy,1个男性
1girl,1个女性
```

## 许可证 (License)

本项目采用 [Apache-2.0 OR MIT](./LICENSE) 许可证。

This project is licensed under [Apache-2.0 OR MIT](./LICENSE).
