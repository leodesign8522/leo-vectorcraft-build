# Core Bridge 0.14.9 / VectorCraft 0.22.0 验证记录

## 修复

- 修复 V 整组选中后按 A，从子路径节点直接拖动会缩成单点选择、拉坏图形的问题。选中父组现在涵盖其子路径；节点、曲线或填充直接拖动保持整个组及嵌套组刚性移动。
- 单击并松开仍能进入子路径、线段或单节点选择，再拖动可正常局部编辑。原有 Alt 单击逐层选组、Alt 拖动复制、撤销重做保留。
- Shift＋空格不触发旋转、平移或缩放，也不借用视图导航光标。单独空格保留原生平移，显式 Rotate View 工具保留。按键组合从画布空闲状态开始，不会创建图形。
- 保留 0.14.8 的 I 吸管文字模式复制、状态高亮、默认贴合、网格优先，以及字形边界对齐、节点、移动、缩放、旋转和黄色延长线吸附。

## 测试与来源

- 构建源：`fbe1bfc3270a9ea2509dfff357b5fdbdfadfbf5c`；官方 v0.22.0：`654749c16eb20b6d3182a96defb66e11d1a9649a`。
- [专项自动回归](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42483357333)成功：3 项新整组/嵌套组/单击编辑/复制测试；77 项原生白箭头回归，53 项几何、17 项对象移动、83 项选择、14 项变换、11 项字形、6 项文字模式及 56 项原生 AI 编辑数据筛选通过（筛选有重叠，不相加）。
- [Windows x64 构建及安装校验](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42483357316)成功。5 项 egui 状态/导航测试验证 Shift＋空格无视图变化、无绘图、无旋转刻度，单独空格平移正常；其他原有兼容性检查通过。
- 同一份实际 EXE 用真实 A 字母快捷键与指针事件核验 30 种情况：整组/嵌套组 × 节点/填充/曲线 × 普通/智能吸附/网格/拖动中 Shift 约束/Alt 复制。所有成员刚性移动，Alt 原件不动，撤销重做通过。单击再拖动仍可单点变形，内部图形不变。回归曾在旧版 0.14.8 正确失败，捕获原组选择被缩为子路径的问题。
- 同一份 EXE 的 I 吸管、旧文件、8 种网格优先、6 种字形对齐、5 种文字渲染/保存/编辑、6 种 R 旋转、8 种边框缩放及跨对象双向延长线回归通过。

## 安装包

从 **Leo Design VectorCraft 0.22 Bridge 0.14.9** 新快捷方式启动，在“关于”核对 **Core Bridge 0.14.9 / Snapping Engine 1.9**。独立目录安装，旧窗口不会自动变成新版，请先保存旧窗口文档。

- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42483357316/artifacts/13619559227)。
- [便携 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42483357316/artifacts/13620350132)。
- 主程序 SHA-256：`8f10467823c36ac8019beaad3518734f0c55ba0747b220765f2fa826e93a2b31`。
- 安装器 SHA-256：`8e7bc89144cc07fad030fb0645bd6f5e388dc4d9f583b80d32ca9ee31385400c`。
- 已核验 GitHub artifact digest、BUILD_COMMIT、UPSTREAM_COMMIT、Windows x64 PE、程序 --version 和随包 SHA256。
