# Leo Design Core Bridge 0.14.0 验证记录

> 历史记录：0.14.0 的测试未覆盖超过 20,000 个锚点后的目标漏收集。该问题已在 [0.14.1](VERIFIED_RELEASE.md) 修复。

适配官方 VectorCraft 0.22.0，Windows x64。安装器和便携版使用同一个已测试的主程序。

## 交付与来源

- [全部回归与正式 EXE 构建：成功的 build job](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42346832709/job/128326307382)
- [扩展节点回归：成功](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42346832731)
- [安装器生成及实际安装校验：成功](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42358116808)
- [安装包 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42358116808/artifacts/13568810730)
- [便携包 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42346832709/artifacts/13565086229)

官方源码：`654749c16eb20b6d3182a96defb66e11d1a9649a`。
主程序构建源码：`f96207bf7203f5b3d9054a581402054c83cf1587`。
安装器配置提交：`614ddd065369de5296dc0548bebab223f392e026`。

最初的安装器步骤因 Windows 图形程序版本输出未被捕获，缺少 VERSION.txt 而失败。修复的是打包元数据捕获；重新打包严格锁定上述已通过全部测试的 EXE 哈希，没有替换未经测试的主程序。

## 修复与自动回归

节点拖动、对象、多选及编组移动已接入 Leo Snapping Engine。支持真实节点、有限直线、Bezier 路径和画板四边；捕获 8 屏幕像素、超过 12.8 屏幕像素释放，随缩放换算。编组使用统一平移，内部结构不变。

Shift 沿锁定方向求真实交点；按下 Shift 不再先取消被拖节点。Shift 单击仍在未拖动的释放时切换选择。Alt 复制使用一致的几何捕获入口，原对象不变。几何优先，无几何命中时保留网格/像素回退。显式垂直和切线选项保留；节点不把单纯轴向或无限延长线当作真实几何命中。

通过的本次回归包括：

- 73 项原生 Direct Selection 交互测试，以及节点/路径/Bezier、缩放、捕获释放、小数且不在网格上的边缘、Shift+Alt 测试。
- 真实文档会话中的节点 Shift 拖动、整条路径刚性移动、Shift 单击、多选/编组移动、一次提交及撤销/重做。
- 原生参考线、文字吸管、动作快捷键、文档信息、Live Corner、AI/SVG/PDF 字体、ICO/图像导出和已选线段复制/剪切回归。
- 安装器静默安装成功，安装后的 EXE 与已测试 EXE 哈希完全一致；版本、独立设置标记及许可文件存在。

这是本次配置的专项回归结果，不代表运行了仓库全部测试或对所有设计文档进行了人工验收。

## 本机程序复测

使用隔离副本和生成图形，通过程序原生控制接口执行实际工具事件，没有改动原有安装或用户设计文件。

| 情形 | 官方 0.22.0 基线 | 本次主程序 |
| --- | --- | --- |
| 智能参考线与网格同时开启，编组右边缘吸到画板 500 | 停在约 498.8976 | 精确到达 500 |
| 编组内部结构、撤销与重做 | 网格关闭时可通过 | 网格开启时通过 |
| Shift+Alt 编组复制 | 网格关闭时可通过 | 网格开启时通过，原对象不变 |
| Shift 拖动已选节点到斜线路径 | 节点仍为 (380,340)，选择清空 | 节点到 (403.3333333333333,340)，选择保留，另一个节点不动 |

## 校验值

安装器 EXE SHA-256：
`7f38d32ba6cc044180753d3b649bd026390c11b88321bfe9630ea0fe43ca66e2`

安装器 ZIP SHA-256：
`169249b4200e4c2a5d7ae3e8ddc8b2d5e55557e17f61f1b24bbdf44acab5fa8d`

主程序 EXE SHA-256：
`300639ffecb3b49c90364b9525f6dfe5260f20ec8904c20a45715d8a506b1983`

便携 ZIP SHA-256：
`b0675ce8d27fb71c74c27cb6ab6ac78cdfee8a9c6c8e7d4a00e66c700feb6d68`

## 启动

安装器默认装入独立的用户目录，保留官方版本。安装后从“Leo Design VectorCraft 0.22”启动，在“关于”中核对：

`Leo Design Core Bridge 0.14.0 / Snapping Engine 1.3`
