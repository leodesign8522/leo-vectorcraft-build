# Leo Design Core Bridge v0.10.6 / VectorCraft v0.21.0

日期：2026-11-04。官方基线：`11862f61198c8f62c0b22b6c9a331cc5950ac98d`。

## 分开交付

1. Core Bridge 默认版：无饼菜单模块、菜单项、键盘/鼠标拦截。
2. Leo Design Pie Menu v0.7.4：单独可选安装器，需要上述配套 Core；卸载恢复配套 Core，其他增强保留。

当前插件接口没有画布输入或面板扩展接口；可选安装器切换同版本增强 EXE，不是通用 WASM/DLL。
默认 Core 仅保存旧菜单 JSON 设置，不执行它。设置不会被安装器删除；以后重装可选包可继续读取。
如果主程序已经升级，可选包卸载不会用旧 Core 覆盖新程序。

## 保留的增强

- Ctrl 选组内对象、整组锚点移动、彩色辅助线。
- AI 原始编辑数据导入及来源报告、PDF/SVG 字体兼容、中文段落与 Ctrl+Enter。
- 截图粘贴、JPEG/PNG/ICO 导出、面板尺寸与快捷设置。
- 工作区/快捷键 JSON 导入导出；官方原生框选路径段。
- A 工具 Shift 拖点保留已选锚点；页面水平、垂直、45°方向均可吸附真实边交点；原线段延长线有青色预览与琥珀色吸附状态。

## v0.21 兼容调整

- 截图粘贴由原生 CF_BITMAP / GetDIBits 快照补足 PNG、DIB 解码失败回退；Ctrl+V key-up 兜底记住前一帧修饰键状态。
- 面板尺寸字段保留原生 Appearance 瞬时弹出面板字段和默认值。
- 官方新增工具栏、AI 导入、PDF 颜色输出等原生更改不回退。
- 两套补丁链均在干净官方源码上应用；craft-ui 按新版 Cargo.lock 固定提交校验。

## LC-026 新增交互\n\n- Shift 在 A 工具中固定按页面坐标约束水平、垂直和 45°方向，不随 General > Constrain Angle 旋转。\n- 延长线只从原线段移动端点向前识别；预览为青色虚线，吸附成功为琥珀色虚线并显示命中点与标签。\n\n## 自动验证

默认无菜单钩子检查、旧配置 JSON 往返；可选菜单交互回归；曲线、AI 导入、字体、截图、导出、选边与工作区指定回归。
Windows 可选安装器测试：不兼容程序拒绝、安装后精确切换、卸载后 Core 字节一致、配置保留、后续更新保护。新增 Shift 水平/垂直/45°吸附、延长线预览/命中状态与剪贴板候选解码专项回归；PixPin 实机仍需用户验收。
实际用户 GUI、复杂文件、安装环境仍需验收，自动结果以 Actions 为准。

历史 ICC 工作空间设置持久化、完整自由工作区缺口未宣称补齐。

## Current v0.10.6 verification gate

- Build target: VectorCraft v0.21.0, Core Bridge v0.10.6, optional Pie Menu v0.7.4.
- Required automated checks: Shift-click/Shift-drag selection, 45-degree edge intersections, arbitrary-angle line-extension intersections, release-only Ctrl+V, PNG/DIB/native CF_BITMAP fallback, and paired Pie install/remove.
- A successful build is not PixPin GUI acceptance. Confirm an actual PixPin screenshot paste on Windows before marking LC-025 fully accepted.



## v0.10.6 — LC-027 / LC-028

- A 工具选中单条或多条线段，Ctrl+C / Ctrl+X 提取所选几何，不再复制整个闭合路径。
- 保留连接关系、闭合路径跨起点的相邻线段、独立子路径、曲线手柄和对象样式；复制不改变源对象。
- 部分路径提取后解除 live shape 生成器，避免编辑时恢复未复制的边；整对象复制继续使用原生路径。
- Shift 水平/垂直拖点或移动部分选中线段时，Smart Guides 可吸附可见参考线，独立于 Snap to Point。
- 延长线沿用青色虚线预览、琥珀色虚线与命中点；按 Shift 锁定方向时不再显示另一方向的延长线预览。
- 验证门槛：补丁锚点精确匹配；Rust 几何、复制/剪切/粘贴/SVG/撤销与参考线方向回归；两种程序编译；安装器配对校验。
- Windows 上 PixPin、复杂文件与实际操作手感仍需实机验收。
