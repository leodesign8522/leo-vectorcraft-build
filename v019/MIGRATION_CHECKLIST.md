# VectorCraft v0.19.0 → Leo Design Core Bridge 迁移清单

**项目：** `leodesign8522/leo-vectorcraft-build`  
**日期：** 2026-10-31  
**状态：** **v0.8.0 迁移补丁已完成本地源码应用检查，Windows 编译与功能测试待运行。** 不要将 v0.18 Overlay Installer 装在 v0.19 官方版上。

## 已确认的版本基线

| 项目 | 固定版本 | 状态 |
|---|---|---|
| 官方软件 | [VectorCraft v0.19.0](https://github.com/storytold/vectorcraft/releases/tag/v0.19.0) | 已发布 2026-10-31 |
| 旧官方源码 | [VectorCraft v0.18.0](https://github.com/storytold/vectorcraft/releases/tag/v0.18.0) | v0.7.5 增强版基线 |
| 当前可回退增强版 | [Leo Design v0.7.5 build #41179333762](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41179333762) | Windows 原生编译、主安装器、Live Corners 与 Pie Menu 测试已通过 |
| 当前稳定旧版 | `main` 上 Leo Design v0.5.7 / VectorCraft v0.17 | 不覆盖 |
| 新隔离分支 | `leo-v019-migration` | **已创建**；目前仍包含 v0.18 补丁，不能直接改名为 v0.19 已适配 |

## v0.19 更新范围与风险

官方 [v0.18...v0.19 变更比较](https://github.com/storytold/vectorcraft/compare/v0.18.0...v0.19.0) 显示有大量上游修改（GitHub API 报告 754 次提交；变更文件列表存在分页/上限，不代表仅有首批列出的文件）。`crates/ui-egui`, `crates/engine`, `crates/doc` 等都是我们植入功能的区域，因此必须重新做源码锚点检查。

官方版本部分变化可能与我们的补丁重叠，例如 Export raster pixel width/height、单锚点和层操作、网格相关细节、色彩及工具输入改进等；**先确认原生能力再选择是否保留 Leo Design 补丁**，避免重复功能彼此干扰。

## 按风险划分迁移顺序

| 阶段 | 功能编号 | 动作 |
|---|---|---|
| 0 | 官方源 | 仅编译未修改的 v0.19.0 Windows x64，固定 Rust/egui/craft-ui 依赖，建立原版对照 |
| 1 | LC-001/002/003 | Smart Select、整组锚点移动、彩色 Smart Guides；确保新上游输入/工具变化未覆盖 |
| 2 | LC-004/005/006/007/008/009 | 对比 ICC 持久化、颜色面板、PDF 文字与 Ctrl+Enter。**LC-004 是已知 v0.18 遗漏，必须重新评估。** |
| 3 | LC-010/011/012/013/014 | Ctrl+V 截图、工作区/快捷键 JSON、自由布局、停靠拖拉尺寸和单位/颜色侧栏。**LC-012 旧功能未完全验证。** |
| 4 | LC-015/016 | 图像专业导出 JPEG/PNG/ICO；对比 v0.19 新增像素尺寸导出行为，确认 ICC/透明度/批量画板 |
| 5 | LC-017/018 | A 工具选边、Live Corners 源锚点联动；保留线段和圆角测试 |
| 6 | LC-019 | Pie Menu 编辑器 v0.3.0、键盘和修饰键右键、快捷甩动、八方向圆形配置、中文功能搜索；重点测试输入焦点与快捷键冲突 |
| 7 | LC-020 | Windows Explorer 缩略图（独立扩展）；验证官方新文档格式是否兼容，不强制绑定主程序版本 |
| 8 | 发布 | Windows 全量编译、功能回归、安装器版本检查、无签名/杀毒结果提示、用户 GUI 验收；仍保留 v0.7.5 回退 |

## 不可省略的验收标准

- 每个 **LC 编号** 填写：`来源版本` / `官方 v0.19 是否已有同等功能` / `迁移代码` / `自动测试` / `Windows GUI 体验` / `最终状态`。
- `cargo build --release --locked -p vectorcraft`、Live Corners、Pie Menu 等自动测试全部通过；ICO/PNG/JPEG 输出验证文件头、尺寸、透明像素和画板数量。
- 对用户关注的操作做真实 GUI 验收：A 框选一条边、Ctrl+V 截图、拖动右侧停靠面板、快捷键呼出饼菜单、点击圆形八方向编辑器、移动圆角源锚点。
- 检查在 Illustrator 中复制中文/英文混排文字至 VectorCraft：字体保留、断行、段落与 CMYK 处理；没有通过的行为不能宣称无缝。
- 必须测试**设置迁移**：已有快捷键 JSON、工作区 JSON、饼菜单 JSON、ICC 文件与用户偏好，不能只确认程序可以启动。
- 检查新官方更改是否已原生提供某个功能，若已提供，优先删去旧补丁而保留用户设置兼容性。
- **不能把 GitHub 的绿色勾等同于功能全部可靠。**

## 发布纪律

1. 在 `leo-v019-migration` 开新 `v019/` 补丁和 workflow；上游 ref 严格钉死在 `v0.19.0`。
2. `leo-v018-migration` 的原有 Windows 产物不能删除，直到官方 v0.19 增强版实机通过。
3. 覆盖安装器必须验证官方版本，阻止 v0.18 安装器覆盖 v0.19 和反向覆盖。
4. 配置文件优先保持独立；用户日常仅导出工作区/快捷键 JSON 可以，但不能用这两份文件替代完整 ICC/饼菜单资源的安全方案。
5. 每完成一项功能，更新 [LEO_DESIGN_FEATURE_REGISTRY.md](../LEO_DESIGN_FEATURE_REGISTRY.md)；新功能另设唯一 LC 编号。未完成项不得标绿色。

## v0.8.0 迁移记录

- 上游固定为 `v0.19.0` / `bcebabe3f8c23a376ac68e9e6726590af278f7bb`。
- v019 使用独立补丁，不修改 v018 回退版本。
- 所有 Core Bridge 补丁已在干净的官方 v0.19 源码上顺序应用通过。
- 修复 ICO 格式注册锚点：保留官方新增 PCX 格式。
- 适配 Selection / Direct Selection 方法拆分，保留官方 selected-source 命中范围。
- Live Corners 迁移保留官方 outermost 过滤，避免父子对象重复移动。
- LC-017 使用官方原生 crossed_segments + select.segments，旧框选补丁不再重复注入。
- 饼菜单适配 canvas_input / canvas_cursor 拆分，呼出时阻止画布工具收到同一输入。
- craft-ui 补丁按 Cargo.lock 精确选择依赖提交，避免改错缓存中的另一个版本。
- 覆盖安装器只接收 v0.19.0；无法读取版本时停止。
- LC-004 ICC 持久化仍未恢复；LC-012 完整自由布局仍待核对。未宣称所有历史需求均已完成。
- Windows GUI、实际安装、真实文件和配置迁移尚未验收。
