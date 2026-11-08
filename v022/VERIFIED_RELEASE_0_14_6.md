# Leo Design Core Bridge 0.14.6 验证记录

VectorCraft 0.22.0 / Windows x64。保留 Leo Snapping Engine 1.7。

## 贴合字形选框

选中文字，在“字符 / Character”面板点击 **贴合字形选框**；也可从 Type 菜单运行同名命令。再次点击恢复原选框。

字符面板较矮时请拉高面板；也可以直接使用“文字 / Type → 贴合字形选框”。

- 对点文字和多行段落文字测量实际排版后的字形轮廓，排除字体的 em 空白与段落框的多余空白。
- 只切换选择/变换控件的测量方式。原文字位置、字号、段落框、换行、文字内容和可编辑状态保持不变；渲染和导出继续使用原排版。
- 贴合模式下 V 选框手柄缩放整段文字，不把紧选框误当作段落排版框而重新换行。用 Type 工具编辑时仍使用真实段落框。
- 支持旋转文字、中文、升降部、Ruby 和原生字形图像/彩色字形测量；空白文字无字形时保留原框。
- 每个文字对象独立保存开关，支持多选、普通组内文字、撤销重做和文件保存重开。旧文件默认保留原选框。
- 有透视、几何变形或复杂剪切/自身外观的对象保留原生选框，避免用原字形错误表示变形后的结果。描边边界按现有 Use Preview Bounds 选项计量。

## 自动化与实际程序验证

- 官方源码固定为 `654749c16eb20b6d3182a96defb66e11d1a9649a`。
- 构建源码：`5b161abd8212d7b7a9eff7e2a2b38d90cd314367`。
- [专项回归](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42460703777)：成功；8 项文字回归通过，覆盖点文字、中文、多行、旋转局部轮廓、空白回退、序列化/旧文件、编辑后更新、数值缩放、真实 V 手柄、编组/预览边界和撤销重做。
- 53 项吸附引擎、17 项对象移动、77 项 Direct Selection、83 项 Selection、14 项原生变换工具筛选通过（筛选有重叠，不相加）。
- [完整 Windows 构建及安装验证](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42460703778)：成功；保留文字字体兼容、吸管、Actions、Document Info、剪贴板、AI/PDF、导出和工作区回归。
- 本机隔离便携程序：单字母、多行段落、中文、旋转和空白五种合成文档，启用前后渲染 PNG 完全一致；对象仅增加开关；保存重开、继续编辑及撤销重做通过。
- 实际字符面板按钮点击和选框截图检查通过。保留 R 旋转、V 中间手柄及延长线原生工具检查通过。

## 安装包与哈希

从 **Leo Design VectorCraft 0.22 Bridge 0.14.6** 快捷方式启动，避免继续操作旧版窗口。安装目录独立，快捷方式使用 `--new-instance`。

- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42460703778/artifacts/13610064059)。
- [便携 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42460703778/artifacts/13610516699)。
- 主程序 SHA-256：`00fe625a105901e13f9b6444f1dc35c494db76be1577eb23c6512def280696e8`。
- 安装器 SHA-256：`58f970e578ee61c2c5d562de6d40bb156bf12e58832d02a3ae9f9bf2d9c79cb2`。
- 已核对 GitHub artifact digest、BUILD_COMMIT、UPSTREAM_COMMIT、Windows x64 PE、`--version` 和随包 SHA256。

原版文件格式会保留文字可编辑内容；本扩展的贴合选框开关由 Leo Core Bridge 版本读取。
