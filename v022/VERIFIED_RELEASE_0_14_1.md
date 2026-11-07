# Leo Design Core Bridge 0.14.1 验证记录

适配官方 VectorCraft 0.22.0，Windows x64。修复 0.14.0 的大文档漏吸附。

## 根因与修复

旧目标收集器耗尽 20,000 个锚点预算后，不再收集后续路径。后添加的目标线因此不参与吸附，普通和 Shift 拖动都会失败。

现在保存拖动开始时的完整路径快照，按指针或移动选区附近的范围查询真实节点、有限直线和 Bezier。对象对齐边界也不再随锚点预算被截断。捕获锁保存实际路径段，附近候选列表变化不会导致锁定跳到另一条线。8px 捕获、12.8px 释放、Shift 方向约束和 Alt 复制保留。

## 来源与构建

- 官方源码：`654749c16eb20b6d3182a96defb66e11d1a9649a`。
- 主程序构建源码：`427b1c93cd78960034fd560ff571d3b2b4533754`。
- 追加 Bezier 测试提交：`bccec40aa82741c3bb9e7b137ff4ed1143cb007d`，仅增加测试，生产引擎代码相同。
- [完整回归、Windows 编译和安装校验](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42362365957)：成功。
- [扩展大文档 Bezier 专项回归](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42363539932)：成功。
- [安装包 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42362365957/artifacts/13570649176)。
- [便携包 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42362365957/artifacts/13570947701)。

## 验证结果

- 专项筛选分别通过 31 项引擎/节点测试、17 项对象移动测试、74 项 Direct Selection 测试；这些筛选有重叠，不相加。
- 4 项真实文档会话回归通过，包括大文档节点拖动的两种 Shift 状态、提交、撤销与重做；其余配置的 Core 回归通过。
- 20,004 个合成背景锚点之后添加源节点、直线和 Bezier，检查实际有限几何命中；Bezier 覆盖 0.25、1、4 三种缩放。
- 本机合成大文档：节点与整组移动在拖动中、松开后均命中；组内两个对象保持统一平移，撤销/重做通过。
- 本机原生程序：网格开启时的画板吸附、组内结构、撤销/重做和 Shift+Alt 复制通过，原对象保持不变。
- 原文档恢复副本仅在本机测试：相同缩放下，0.14.0 按住与不按 Shift 均复现间隙；0.14.1 两种状态在拖动中均命中目标直线，提交、撤销、重做通过，另一个节点和目标线保持不变。用户文件与实际绘图坐标没有上传。
- 安装器实际安装成功，安装后的主程序哈希与测试程序一致。

本机检查通过程序原生控制接口发送工具事件。以上是配置的专项回归与上述文档副本结果，不代表对所有文档或全部操作的保证。

## SHA-256

| 文件 | SHA-256 |
| --- | --- |
| 主程序 EXE | `92e0b7029321f230cd432df40ee3ff61f2f6a2924f8b27ed01521b2fa8122ada` |
| 安装器 EXE | `33c572180a036ecf4c95dce4df565e5abc91998f3ae6b72c771b244061fb3b41` |
| 便携 ZIP | `5b09504a129e26a1d4b855ea8ff746928c4026eed2a03a85ca535d63a68037a0` |
| 安装器 ZIP | `7d24c6b73f77d06c80064cbc0a385d3064e0f54bb516e31987f77e7fb1c253e0` |

## 安装与启动

先保存旧窗口中未保存的文档，再安装并从 **Leo Design VectorCraft 0.22 Bridge 0.14.1** 快捷方式启动。该快捷方式强制启动新实例，避免激活仍在运行的旧版。

默认目录：`%LOCALAPPDATA%\Programs\Leo Design\VectorCraft-0.22-Bridge-0.14.1`。保留旧版与官方目录。

在“关于”核对：`Leo Design Core Bridge 0.14.1 / Snapping Engine 1.3.1`。

[0.14.0 历史测试记录](VERIFIED_RELEASE_0_14_0.md)没有覆盖此次大文档情形，应结合本次修复记录阅读。
