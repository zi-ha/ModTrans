## ModTrans UI 重构计划(按你的方案实施)

### 一、全局框架(`src/App.vue`)
- 侧边栏收窄至 **~100px**,顶部加"ModTrans"品牌名(小字)
- 导航三项:翻译(网格图标)/ 合并(图层图标)/ 设置(齿轮图标),路由改为 `/translate`、`/merge`、`/settings`
- **视觉规范整体替换**:背景 `#F7F8FA`、主色 `#1677FF`、成功 `#52C41A`、错误 `#FF4D4F`、警告 `#FA8C16`,hover/选中浅蓝 `#E6F4FF`,边框统一 `#EBEDF0`(替换现有 #2563EB/#16A34A/#EF4444/#EFF6FF 等)
- 弱化卡片与分割线:共享样式保留,页面内改为"标题 + 内容 + 间距"结构

### 二、工作台重构(`src/views/WorkspaceView.vue`)
**删除顶部步骤条**,改为按 `store.currentStep` 状态驱动的 4 个页面(现有状态机复用,无步骤条展示):

1. **导入页**:紧凑拖放条("拖入 .jar 文件" + [选择文件],不再大面积上传框)+ 文件列表 + [开始提取]
2. **翻译页**:
   - 未开始:Mod 信息行(模组勾选、文本条数)+ [开始翻译]
   - 翻译中:进度条 + `43 / 89` + 当前原文/译文对照 + [暂停] [停止]
   - 完成:✓ 翻译完成 N 条 + **[查看校正]** **[导出资源包]**(直接导出 = 自动跳过校正标记)
   - 顶部"← 更换 Mod"可回导入页
3. **校正页**(核心,左右对照布局):
   - 顶部一行:搜索 + 模组筛选 + 批量替换
   - 左右对照表:原文列(含 key 小字)| 中文列(点击直接编辑,Enter 保存、Esc 取消)
   - 底部:统计 + **[导出资源包]**;左上"← 返回翻译"
4. **导出页**:资源包名称 / Minecraft 版本 / 作者 + 包含 Mod 勾选 + 统计 + [导出] + 成功提示;左上"← 返回校正"

### 三、合并页(`src/views/MergeView.vue`)
标题"合并资源包",紧凑拖放 + 文件列表 + 输出名称 + [合并],视觉同步新配色。

### 四、设置页(`src/views/SettingsView.vue`)
改为单页区块式(去掉顶部 tab):
- **AI 服务**:现有配置下拉 + 表单(名称/提供商/API地址/模型/Key)+ 测试连接 + 保存
- **翻译规则**:✓ 保留变量 ✓ 保留颜色代码 —— **真实生效**:勾选保存到设置,翻译请求时前端把规则文本拼入提示词(不是死 UI)
- **词典管理**:现有术语库
- **历史**:折叠区块保留(数据有实际记录价值)

### 五、Rust 最小改动(`src-tauri/.../commands/store.rs`)
- `AppSettings` 新增 `translation_rules: Vec<String>` + `#[serde(default)]`,兼容旧 settings.json;其余命令不动

### 六、类型与路由
- `src/types.ts`:AppSettings 加 `translation_rules`
- `src/router/index.ts`:`/workspace` 更名 `/translate`(redirect 同步)

### 七、验证
- `npm run build`(类型检查 + 构建)+ `cargo check`(Rust 改动)

### 文件清单
改:`App.vue`、`WorkspaceView.vue`、`MergeView.vue`、`SettingsView.vue`、`router/index.ts`、`types.ts`、`src-tauri/src/commands/store.rs`