# ModTrans - MC 模组译途

桌面端 Minecraft 模组翻译工具，支持 Jar 导入、AI 翻译、一键生成汉化资源包。

## 功能

- **Jar 提取** — 拖入 .jar / .lang / .json 自动提取英文语言文件
- **AI 翻译** — 支持 OpenAI / DeepSeek / 智谱 / 通义千问，内置 MC 术语库
- **人工校正** — 表格化校对，双击编辑，批量替换，区分原生/模组词条
- **资源包生成** — 一键生成 .zip 汉化包，支持 1.12 ~ 1.21 各版本
- **多包合并** — 拖入多个汉化包合并为一个

## 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 技术栈

- 前端：Vue 3 + TypeScript + Vite
- 后端：Rust + Tauri 2
- 翻译：OpenAI 兼容 API

## License

MIT
