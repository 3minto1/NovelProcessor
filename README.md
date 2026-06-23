# NovelProcessor

基于 AI 的小说章节验证与审查桌面工具。导入 TXT 小说文件，自动识别章节标题，使用 AI 验证标题合理性并审查修正内容。

## 功能

- **导入 TXT 小说**：支持 UTF-8 和 GBK 编码，自动识别章节目录
- **AI 验证章节标题**：批量判断章节标题是否合理（有效/无效），支持并行处理
- **AI 审查内容**：修正错别字、删除无关内容（作者笔记、广告等）、修复语法问题
- **对比视图**：原文与审查结果并排对比，支持差异高亮、全局搜索、手动编辑
- **单独审查**：可对单个章节重新发起 AI 审查
- **导出**：导出处理后的小说文本或章节目录
- **多模型支持**：OpenAI 兼容接口、Google Gemini，内置模型推荐

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | React 18 + TypeScript + Vite |
| 状态管理 | Zustand |
| 后端 | Rust |
| 数据库 | SQLite (rusqlite) |
| AI 接口 | OpenAI 兼容 / Gemini |

## 开发环境

### 前置要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.85
- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建

```bash
npm run tauri:build
```

### 测试

```bash
npm test
```

### 打包 Portable

```bash
npm run package:portable
```

## 使用方法

1. 启动应用，点击「导入 TXT」导入小说文件
2. 在左侧选择模型配置，填写 API Key
3. 点击「验证章节」让 AI 判断章节标题是否合理
4. 标记为无效的章节可以删除或手动调整
5. 点击「审查内容」让 AI 修正错别字和语法问题
6. 在对比视图中查看原文与审查结果的差异
7. 确认无误后点击「导出」保存处理后的小说

## 设置说明

| 设置项 | 说明 | 默认值 |
|--------|------|--------|
| 每批次章节数 | 验证时每批发送的标题数；审查时每组章节数 | 30 |
| 审查并发 | 审查时同时运行的 AI 请求数 | 10 |

审查时章节分组逻辑：每组章节数 ÷ 并发数 = 每个请求的章节数。例如 100 章 / 50 并发 = 每个请求 2 章。

## 项目结构

```
NovelProcessor/
├── src/                    # 前端 React 代码
│   ├── components/         # UI 组件
│   ├── config/             # 模型推荐配置
│   ├── hooks/              # React Hooks
│   ├── store/              # Zustand 状态管理
│   ├── types/              # TypeScript 类型定义
│   ├── App.tsx             # 主应用组件
│   ├── tauriApi.ts         # Tauri 命令封装
│   └── styles.css          # 全局样式
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── ai/             # AI 接口（OpenAI/Gemini）
│   │   ├── commands/       # Tauri 命令处理
│   │   ├── db/             # 数据库 Schema
│   │   ├── repositories/   # 数据访问层
│   │   ├── services/       # 业务逻辑
│   │   ├── text/           # 文本处理（编码检测、章节分割）
│   │   └── lib.rs          # 入口
│   └── Cargo.toml
├── scripts/                # 构建脚本
└── package.json
```

## 下载

从 [Releases](https://github.com/3minto1/NovelProcessor/releases) 页面下载最新版本。

## License

MIT
