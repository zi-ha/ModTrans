## 目标
优化每个界面布局与交互逻辑,剔除经后端代码核实的死功能与冗余元素。仅做前端重构 + 1 处最小 Rust 兼容改动。

## 一、冗余清理(逐项核实过后端)

| 位置 | 冗余项 | 依据(后端证据) |
|---|---|---|
| 生成步骤 | 整个"高级选项"折叠(生成 pack.mcmeta / UTF-8 修复 / 合并原语言文件 3 个 checkbox) | `generate_pack` 命令无这些参数,pack.mcmeta 恒生成 |
| 翻译步骤 | "语言"下拉(固定"简体中文") | 无任何逻辑 |
| 翻译步骤 | "高级设置"折叠(只读 API 地址) | 只读展示无操作 |
| 设置页 | "文件"tab(仅默认输出目录) | `default_output_dir` 后端存储后无任何命令使用,生成/合并均用保存对话框 |
| 前端 | 硬编码的大段翻译系统提示词(新配置默认值) | 与后端 `TRANSLATION_SYSTEM_PROMPT` 重复;新配置默认改为空,由后端系统提示词兜底 |
| 前端 | SettingsView 本地 `providerPresets` | 与后端 `get_provider_presets` 命令重复,改为调用后端命令做单一来源 |

## 二、布局优化

**工作台各步骤**(统一"工具栏行 + 内容区 + 底部操作行"结构):

1. **① 导入**:文件列表卡内合并"已选 N 个文件"标题、列表和"开始提取"按钮,去掉独立 bottom-bar 分隔
2. **② 翻译**:顶部一行工具栏(模组勾选 + API 配置下拉,未配置时显示提示条);进度区合并为一个大卡片(进度条、百分比、已完成/剩余统计、当前词条、开始/暂停/停止、日志);删除"翻译设置"折叠(API 选择已上移工具栏);底部"进入校正 →"按钮
3. **③ 校正**:搜索、模组筛选、批量替换合并为一行工具栏;表格卡保留;底部统计精简为一行 + "进入生成 →"按钮;"未确认"tab 改名"未翻译"
4. **④ 生成**:两个卡片(Mod 选择 + 资源包信息)合并为一个"资源包信息"卡,内嵌包含 Mod 勾选;底部统计 + "生成 ZIP"按钮;删除高级选项折叠

**合并页**:MC 版本列表与生成页统一(11 个版本);输出文件名/版本/开始合并按钮整合进文件列表卡,去掉独立 bottom-bar

**设置页**:3 个 tab(AI 服务 / 术语库 / 历史);AI 服务卡片布局保留(结构已合理);新增配置默认提示词留空;保存成功提示抽为共享 `src/utils/toast.ts`(消除两处 document.createElement 重复代码)

## 三、Rust 最小改动
- `src-tauri/src/commands/store.rs`:`default_output_dir` 加 `#[serde(default)]`,兼容前端不再发送该字段(settings.json 旧文件不受影响)
- 后端其余命令不动

## 四、文件清单
- 改:`src/views/WorkspaceView.vue`、`src/views/MergeView.vue`、`src/views/SettingsView.vue`、`src/types.ts`(移除 default_output_dir)、`src/stores/app.ts`(移除未使用的 defaultOutputDir)、`src-tauri/src/commands/store.rs`
- 新增:`src/utils/toast.ts`

## 五、验证
- `npm run build`(类型检查 + 构建)
- `cargo check`(Rust 改动验证)