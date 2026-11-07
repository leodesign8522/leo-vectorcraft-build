# Leo Design Core Bridge 0.14.2 验证记录

官方 VectorCraft 0.22.0，Windows x64；Leo Snapping Engine 1.4。

## 本次升级

A 工具直接拖动单个节点时，可沿该节点原有直边双向延长、向回缩短，也可跨过另一端继续保持共线。闭合图形同样适用。原边方向在拖动开始时冻结，不随拖动预览改变；曲边不被当作直边。

- 捕获 8 屏幕像素、释放 12.8 屏幕像素，随缩放换算。
- 只检查该节点相邻直边，最多两个方向，不新增全图延长线搜索。
- 共线命中显示青色虚线、圆标和 `extension` 标签；没有命中时不显示。
- 真实节点、有限直线/Bezier 与画板边缘的几何命中优先；未捕获时保留网格/像素回退。
- Shift 保持原有方向约束，不重复叠加延长线捕获。普通对象、编组和 Alt 复制保留原有行为。
- 保留 0.14.1 的大文档目标收集修复。

## 使用

开启 **视图 → 智能参考线**，用 **A 白箭头选中一个节点后直接拖动，无需按 Shift**。靠近原边方向时出现青色共线标记；拖离释放距离后可以自由改变方向。

## 构建与回归

- 官方源码：`654749c16eb20b6d3182a96defb66e11d1a9649a`。
- 本次主程序源码：`893781382e36117022d990d086b3d7ecbfbe3575`。
- [完整 Windows 构建、配置的保留功能回归及实际安装校验](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42369030117)：成功。
- [吸附专项回归](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42369030064)：成功；34 项引擎/节点筛选、17 项对象移动筛选、75 项 Direct Selection 筛选均通过，筛选有重叠，不相加。
- 5 项真实 Session 回归通过，包括网格开启时双向延长/回缩的提交、撤销与重做。
- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42369030117/artifacts/13573882985)。
- [便携 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42369030117/artifacts/13573917687)。

## 本机程序验证

通过程序原生控制接口发送实际工具事件，使用生成的闭合四边形及 20,004 个背景锚点，用户绘图和截图没有上传到 GitHub。

| 场景 | 结果 |
| --- | --- |
| 沿原边向外延长 | 拖动中及松开后共线，其他三个节点不变 |
| 沿原边向回缩短 | 拖动中及松开后共线，其他三个节点不变 |
| 拖过原边另一端 | 共线，提交、撤销与重做通过 |
| 12.7px 保持、12.9px 释放、再次捕获 | 实际程序通过 |
| 先拖离原方向，再拖回 | 捕获拖动开始时的原方向，不追随临时预览方向 |
| 大文档节点/整组移动 | 普通与 Shift 两种状态通过，组内结构及撤销重做通过 |
| 网格开启、画板吸附、Shift+Alt 复制 | 通过，原对象不变 |
| 既有文档本地恢复副本 | 普通/Shift 几何吸附仍通过，目标及固定节点不变 |

0.14.1 的同一合成闭合图形负对照停留在指针坐标，没有共线捕获；本次回归可检测升级缺失。最初测试还发现 Shift 平行时的重复反馈，已修正并由上述成功运行覆盖。

以上是配置的专项测试与本机所列场景，不代表完成所有文档的性能压测或全部人工鼠标操作验收。

## SHA-256

| 文件 | SHA-256 |
| --- | --- |
| 主程序 EXE | `e548866bfc3e05f9f749d50aeef96b10df7d7eca64d4b7485f5354de484b3399` |
| 安装器 EXE | `ff2893aebb82e46f4bd75af55425c7a49c5ec7a8221daaf0aa68575e731c2fd9` |
| 便携 ZIP | `86f96e45b34456a5ccc9a24e5d95d3ed83ac73745b7dc67b97a2df1bcc226cce` |
| 安装器 ZIP | `fa0568c4f9e2f4035a3753a1102dfba0605c8e74c3db39501bea993fe46ec466` |

## 安装与启动

先保存当前窗口中未保存的文档，再安装并从 **Leo Design VectorCraft 0.22 Bridge 0.14.2** 快捷方式启动。快捷方式启动新实例，避免激活旧版窗口。

默认目录：`%LOCALAPPDATA%\Programs\Leo Design\VectorCraft-0.22-Bridge-0.14.2`；保留旧版本和官方程序。

关于显示 `Leo Design Core Bridge 0.14.2 / Snapping Engine 1.4`；`--version` 可识别同一版本。

[0.14.1 大文档修复历史记录](VERIFIED_RELEASE_0_14_1.md)。
