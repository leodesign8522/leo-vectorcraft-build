# Core Bridge 0.15.0 / VectorCraft 0.22.0 验证记录

## 新增：可编辑 AI 导出

入口：**文件 → 导出 → 导出可编辑 AI（通过 Illustrator）…**。需要本机 Windows Illustrator 和相应字体；本机验证版本为 Illustrator 2026 30.8.1。

- 点文字保持可编辑；多行文字保留整段区域文字框，可继续自动换行。混合字体/字号和文字变换按支持范围映射。
- 保留 RGB/CMYK 文档模式和已测普通填色色值、图层与嵌套图层、编组、隐藏/锁定状态、多画板名称/尺寸/位置。
- 源文档和选择保持不变。缺少字体会明确报错；失败不替换已有目标文件。临时目录结束后清理，Illustrator 原有文档不改动。
- 空文字框保持为空，重新输入后保留原字体，无占位字符残留。
- 原生 AI 导出继续提供；其写给其他软件的编辑数据仍可能将文字转轮廓。新入口通过 Illustrator 原生保存活文字。

## 明确范围

RGB/CMYK 模式和数值不等于完整 ICC 配置继承。当前遇到显式指定 ICC 的文档会停止并说明；导出采用 Illustrator 的工作配置并嵌入输出。

超过 31 字符的全局/专色色板名称、路径文字、串接框、注音/绕排/行内图形、文字自身特殊混合模式或额外外观/蒙版、符号/图案中的文字及预检拦截的高级字符属性暂不支持。复杂段落排版、字体版本差异和跨软件重排可能改变字距或折行，不能称为所有文件无损。Illustrator 可能提示文字布局版本更新。

## 测试与来源

- 构建源：`1ebee090eb9299fd36f7f93c8961ff56c2b02fb0`；官方 v0.22.0：`654749c16eb20b6d3182a96defb66e11d1a9649a`。
- [Windows x64 发布构建与隔离安装校验](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42539984128)成功；6 项新 AI 预检/快照/混合文本/几何/指定 ICC 防丢失测试和全部配置的保留回归通过。
- [吸附专项回归](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42539984145)成功。
- 同一份实际 EXE 从 `file.leoExportEditableAi` 入口导出 RGB、CMYK 测试文档：原文档节点和选择不变，两个画板名称/尺寸保留；Illustrator 检查两层和编组、可编辑点文字与中文整段框、原位置与框尺寸、精确 RGB/CMYK 数值；缩窄框自动重排，编辑、保存、重开后仍可编辑。空框和缺字体失败保护通过。
- Illustrator 脚本补充验证：隐藏/锁定容器、嵌套图层与编组、旋转混合字号/粗体文字、多画板名称；实际 Windows PowerShell 桥接的成功和失败路径均通过。
- 同一份 EXE 的 30 种 A 整组/嵌套组拖动、I 吸管文字模式与旧文件、8 种网格优先、6 种字形对齐、5 种文字渲染/保存/编辑、6 种 R 旋转、边框缩放和跨对象黄色延长线回归通过。测试筛选有重叠，不累加成全量套件。

## 安装包

从 **Leo Design VectorCraft 0.22 Bridge 0.15.0** 新快捷方式启动，在“关于”核对 **Core Bridge 0.15.0 / Snapping Engine 1.9**。独立目录安装；吸附引擎保持 1.9，保留 0.14.9 的增强。

- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42539984128/artifacts/13642950337)。
- [便携 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42539984128/artifacts/13642580838)。
- 主程序 SHA-256：`f4ff70416fd664d80c2cab1df3c91cbed44094d071c561d89b1c3d32c5ce9e53`。
- 安装器 SHA-256：`a8a5c37dec8881f81e5f9c4414bbe711291150de9be7d97f9cb5844e520978fe`。
- 已核验 GitHub artifact digest、BUILD_COMMIT、UPSTREAM_COMMIT、Windows x64 PE、程序版本和随包 SHA256。
