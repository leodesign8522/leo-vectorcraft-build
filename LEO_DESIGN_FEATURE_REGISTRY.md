# Core Bridge 0.14.4 / Snapping Engine 1.6 边框缩放吸附（验证中）

- 中间手柄缩放接入完整目标快照，修复 20,000 个锚点后目标漏收集。
- 几何吸附先于网格和像素回退；保持对侧边、Alt 中心与 Shift 比例约束。
- 四边缩放使用 8px 捕获、12.8px 释放，补充原生工具和会话回归。
- Windows 构建与本机验证待完成。

# Core Bridge 0.14.3 / Snapping Engine 1.5 跨对象延长线与黄色命中（已验证）

- A 单节点支持邻边、当前路径其他直边及其他矢量对象直边的双向延长线；非相邻/跨对象参照只取当前视野内的直边。
- 参照范围随平移缩放更新；已吸住的延长线保持到 12.8px 释放，避免平移画布时突然掉线。
- 亮黄色虚线与描边圆标、深色底“延长线吸附”标签明确区分延长线命中。
- 拖动开始时建立方向/偏移索引，精确投影确认命中；无 20,000 条截断。
- 保留 8px 捕获、12.8px 释放、真实几何优先与原有 Shift 行为。曲线不作直线延长，隐藏对象不参与。
- Windows x64 编译、专项/保留功能回归、安装版本与哈希检查，以及本机原生工具事件验证通过。
- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42385939388/artifacts/13581246846)；[验证记录与 SHA-256](v022/VERIFIED_RELEASE.md)。
- 从 **Leo Design VectorCraft 0.22 Bridge 0.14.3** 快捷方式启动，开启智能参考线，用 A 拖动单节点，无需 Shift。

# 历史交付 Core Bridge v0.14.2 / Snapping Engine 1.4 双向共线升级

- A 工具单节点可沿原有直边双向延长、回缩；同时支持闭合路径和任意角度直边。
- 只冻结该节点相邻直边，最多两个方向，不进行全图延长线搜索；曲边不当作直线延长。
- 8px 捕获、12.8px 释放；实际节点/有限路径命中优先，Shift 方向锁定保持原有行为。
- 命中显示青色虚线、圆标和 extension 标签，与真实路径命中区分；未命中无反馈。
- 保留 0.14.1 的大文档目标收集修复。Windows 编译、配置的回归、安装哈希校验和本机双向共线/释放/撤销重做均通过。
- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42369030117/artifacts/13573882985)；[验证记录与 SHA-256](v022/VERIFIED_RELEASE_0_14_2.md)。
- 开启智能参考线，用 A 拖动单节点，无需 Shift。从 **Leo Design VectorCraft 0.22 Bridge 0.14.2** 快捷方式启动。

## 历史交付 Leo Design Core Bridge v0.14.1 / VectorCraft v0.22.0

- 修复超过 20,000 个锚点后，后新增目标路径漏收集而不吸附的问题；按拖动范围查询完整路径快照，锁定目标不依赖临时候选索引。
- 大文档直线/Bezier、普通及 Shift 节点拖动、对象/整组移动、捕获释放及原生 Direct Selection 回归通过；Windows x64 编译与安装后哈希校验通过。
- 原文档本地恢复副本中，按住/不按 Shift 在拖动中均命中；提交、撤销、重做通过，目标线和另一节点不变。文件和实际绘图坐标没有上传。
- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42362365957/artifacts/13570649176)；[验证记录与 SHA-256](v022/VERIFIED_RELEASE_0_14_1.md)。
- 默认独立目录，使用 **Leo Design VectorCraft 0.22 Bridge 0.14.1** 快捷方式；关于显示 Core Bridge 0.14.1 / Snapping Engine 1.3.1。

## 历史交付 Leo Design Core Bridge v0.14.0 / VectorCraft v0.22.0

- Windows x64 主程序已通过本次全部配置的回归及正式编译；独立安装器已通过实际安装、版本与哈希一致性检查。
- 修复网格/像素优先级绕过几何捕获、Shift 按下取消所选节点、Alt 复制入口不一致；保留无几何命中时的网格回退及真实垂直/切线选项。
- 8px 捕获、12.8px 释放，真实节点/直线/Bezier/对象与画板边缘，多选和编组统一平移，Shift 方向锁定与 Alt 复制。
- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42358116808/artifacts/13568810730)；[验证记录与 SHA-256](v022/VERIFIED_RELEASE_0_14_0.md)。默认独立安装，保留官方版本。
- 帮助/关于及 --version 可识别 Core Bridge 0.14.0 / Snapping Engine 1.3。请从 Leo Design VectorCraft 0.22 快捷方式启动。

## 历史版本 Leo Design Core Bridge v0.13.2 / VectorCraft v0.21.1

- Leo Design Snapping Engine v1.2 与 Core Bridge 合并交付，不再拆分独立 Snapping 安装包。
- 节点与对象移动共享真实几何吸附核心：节点/节点、节点/有限路径与贝塞尔曲线、对象几何点/路径、对象边界对齐及画板四边。
- 初始捕获距离 8 屏幕像素，释放距离 12.8 屏幕像素；缩放时保持屏幕容差。真实节点优先于路径，几何命中优先于轴向对齐。
- 对象移动覆盖普通对象、多对象、编组、Direct Selection 整条路径及 Alt 复制；统一平移，不改变组内结构。Shift 约束使用新引擎的锚点/参考线投影和边界对齐，网格/像素为独立约束。
- 画板边缘作为有限几何目标；辅助线只提示真实命中或有效对齐，不再以提示线冒充吸附成功。
- 文档信息支持已选路径长度和带透明度的物体数。构建与自动化测试结果以本轮 Actions 为准；Windows 鼠标手感仍需用户实机验收。
- 以下为历史构建记录。

## v0.12.0 构建与交付验证（2026-11-05）

- 构建源码：3fabe64005b7a5f7b71814912c44f82a693f76a2；官方基线 v0.21.1。
- [Windows 构建与安装器工作流 #41962681600](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41962681600)：两个 job 均成功。
- Windows x64 release 编译、EXE --version 冒烟检查及所有配置的专项回归通过（0 失败）。
- 回归覆盖文字吸管、动作自定义快捷键和原生 Actions、选择统计、Shift/延长线/参考线/旋转吸附、选中线段和原生参考线复制、活动圆角、AI/SVG/PDF 字体、ICO/图像导出和工作区 JSON。
- [覆盖安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41962681600/artifacts/13387459147)：内含 Leo_Design_Core_Bridge_v0.12.0_Installer_for_VectorCraft_v0.21.1.exe。
- [便携包 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41962681600/artifacts/13387692112)。只选一种交付方式，不叠装 Snapping。
- 安装器编译和版本限制已核对；用户电脑实际安装、真实文件和鼠标手感未在本次自动回归中验收。


---

以下为旧版本与功能精简的历史记录，不能覆盖上方当前交付规则。

## 2026-11-05 功能精简决定（优先于历史记录）

- 用户另确认删除审阅清单 54：移除 Leo Design 快捷键备份菜单（快捷键 JSON 导出、JSON/KYS 导入的便捷入口）；官方快捷键编辑及原生导入/导出能力保留，53 工作区 JSON 导入/导出保留。功能编号保持不变。
- 用户确认删除审阅清单 34～37：外部截图 Ctrl+V 增强、多格式图片备用读取、Ctrl+V 按键兜底、重复粘贴与旧内容防护。
- 当前公共构建入口移除三个 `v021/patch_screenshot_paste*.py` 补丁及其专项测试；历史补丁源码仅留档，不再集成进新构建。
- 截图与外部图片粘贴交回官方实现；所选线段复制/剪切及 AI/PDF/SVG 文字字体兼容保留。
- 后续统一产品名为 Leo Design Core Bridge，吸附模块随包集成；统一打包与官方 0.21.1 适配尚待完成。
- 本次只提交功能精简，不重建安装包。已下载和已安装的旧版本不会随源码变更。

# 当前产品边界（2026-11-07，本节优先于下方历史记录）

- **Core Bridge v0.13.2 / VectorCraft v0.21.1**：统一包含 Leo Design Snapping Engine v1.2。节点吸附与普通对象移动吸附由同一套几何目标、捕获距离、释放锁定及反馈逻辑接管。
- 对象移动覆盖 Selection、Group Selection、Direct Selection 整条路径、多对象/编组平移和 Alt 复制；真实几何目标优先，轴向边界对齐、标尺参考线和 Spacing Guides 已接入对象移动引擎。Shift 约束、参考线、Spacing Guides、网格/像素保持各自明确的处理路径。
- 画板四边纳入有限线段目标；不再仅依赖白色画板视觉边界或原生 Smart Guides 的边界框计算。
- **Pie Menu**：按用户要求停止开发、编译和打包，历史源码归档，已有配置仅作兼容保留。
- v0.13.2 的自动测试需通过后才可交付安装包；Windows GUI/鼠标手感仍需用户实机验收。构建通过不等同于 Affinity 级手感验收。
- 当前安装仍通过完整 EXE 覆盖安装；Snapping 已合并进 Core Bridge，不需要也不应叠装独立 Snapping 包。
- 后续 Snapping 源码变更随 Core Bridge 工作流统一构建；以下旧版本记录只作历史审计，不代表当前功能归属。

- 本轮文字吸管修复：字符尺寸按字符面板显示的有效值取样，再补偿目标文字自身缩放；涵盖字号、显式行距、基线偏移、纵中横位移与水平比例。保持字体、字重、字距、颜色及原有取样选项。新增混合缩放、旋转/反射、局部文字范围、撤销与退化矩阵回归测试；状态以对应构建为准。

- Snapping v0.1.2：旋转端点/角点按固定半径轨迹捕捉真实路径边，取消旋转中的普通轴向对齐红线；命中显示黄色反馈，保留 Shift 45°、自定义轴心和形状尺寸。Core 仍为 v0.11.1，本次不加入任何旋转吸附增强。状态以对应构建与实机验收为准。

---

# Leo Design × VectorCraft — 功能总账（长期维护）

## 当前开发版本：Core Bridge v0.10.6 / VectorCraft v0.21.0（2026-11-04）

- 官方固定基线：`v0.21.0 / 11862f61198c8f62c0b22b6c9a331cc5950ac98d`。
- 安装路径约定（2026-11-03）：后续 Core Bridge 与独立饼菜单安装器默认使用 `C:\Program Files\ArtCraft\VectorCraft`（Inno Setup `{autopf}\ArtCraft\VectorCraft`）；仍可手动选择实际安装目录。已有下载包不会因源码修改而改变。
- 当前分支：`leo-v021-migration`；源码独立保存在 `v021/`，v020 历史源码保留。
- 默认 Core 不编译饼菜单模块、菜单入口或键鼠拦截；只保留旧菜单 JSON 数据。
- 当前开发可选包：Leo Design Pie Menu v0.7.4，仅配套 Core Bridge v0.10.6 / 官方 v0.21.0，单独安装和卸载。
- 其他增强功能继续迁移：智能选择、整组锚点移动、彩色辅助线、文字/字体、AI 导入、截图粘贴、图像/ICO 导出、面板及工作区配置。
- 截图粘贴由 v0.10.4 增加原生 CF_BITMAP / GetDIBits 快照读取与 Ctrl+V 释放事件兜底；仍需 PixPin 实机验收。面板尺寸字段避开新增的原生瞬时弹出面板状态。
- 两种款式的完整补丁应用及源代码隔离检查通过；固定 craft-ui 补丁也通过干净源码检查。
## LC-026 Shift 页面轴吸附与延长线视觉反馈（Core v0.10.5 / 可选 Pie v0.7.4）

- A 工具直接拖动锚点时，Shift 固定使用页面坐标的水平、垂直和 45°方向，不受 General > Constrain Angle 的旋转值影响；三个方向都通过同一套方向约束与真实路径交点吸附。
- 原线段延长只识别移动端点之外的前向射线，避免把原线段内部误判为延长线。
- 指针接近延长方向但尚未进入吸附容差时，显示青色虚线预览；进入吸附容差后，显示琥珀色虚线、命中点圆标和“延长线吸附”标签，与普通洋红参考线明确区分。
- 新增垂直方向交点吸附（含旋转 Constrain Angle 设置）、45°方向交点吸附、延长线预览/命中状态和前向端点判定回归测试。
- PixPin 实机剪贴板仍需用户验收；本次 LC-026 不改变截图粘贴实现。

- 自动测试、Windows 编译、可选包安装/卸载结果以本次 Actions 为准；实际 GUI 仍需验收。
- 旧 ICC 工作空间持久化及完整自由工作区的历史缺口没有被宣称解决。


## LC-024 Shift 锚点拖动与真实边交点吸附（Core v0.10.4 / 可选 Pie v0.7.3）

- 修复 A 工具中 Shift 按下已选锚点时，旧逻辑先执行 toggle、把拖动目标取消选择的问题。已选锚点在按下时保持选中；只有未发生拖动的 Shift-click 才在松开时取消选择。
- Shift 拖动继续按 General › Constrain Angle 的 45°步进约束，并显示锁定方向辅助线；方向线上可吸附到附近真实路径边交点，而不只限于水平/垂直辅助线。
- 原线段延长线识别提前于普通锚点和边吸附，避免附近无关目标把点拉离原方向；交点计算直接与目标路径线段求交。
- 水平、垂直及任意斜角延长方向均参与识别；只接受原固定端点朝向拖动端点的前向射线，并限制吸附位移在容差范围内。
- 新增工具状态机回归：已选点 Shift-drag、Shift 45°方向与斜边交点、任意斜角原线段延长线交点、水平/垂直延长方向。

## LC-025 PixPin / Windows 图片剪贴板读取与 Ctrl+V 兜底（Core v0.10.4 / 可选 Pie v0.7.3）

- v0.10.3 用户实测仍未生效，因此不把旧版自动测试当作 PixPin 实机成功。该版修正旧回退路径：不再依赖 arboard 重新打开 CF_BITMAP，而是在 Windows 剪贴板锁内调用 clipboard-win 的 GetDIBits / get_bitmap，将 GDI 位图快照为 BMP，再与 PNG、DIB 候选依次解码。
- PNG、DIBV5、DIB 候选解码失败后继续尝试原生 CF_BITMAP；全部失败时保留错误数据交给导入层报错，不静默粘贴旧内部剪贴板。
- Ctrl+V 兼容 V key-down 被 egui 吞掉、只收到 key-up 的情况；额外记住前一帧 Ctrl/Cmd 状态，兼容 Ctrl 先松开再松开 V。Paste / PasteImage 事件仍抑制重复粘贴。
- 新增 modifier-transition 与 native CF_BITMAP fallback 单元测试。自动测试通过仍不等于 PixPin 在用户电脑上实机验收；交付后必须以 PixPin 截图 Ctrl+V 实测为准。

## LC-023 截图粘贴兼容补丁（Core v0.10.3 / 可选 Pie v0.7.3）

- 用户实测：Core v0.10.3 安装后 PixPin 截图 Ctrl+V 仍无反应。LC-025 已重新实现 CF_BITMAP 读取和 key-up 兜底；不能把 v0.10.3 的构建测试等同于实机成功。
- 修复已确认的代码缺口：保留 V 按下时的 Ctrl/Shift 状态，防止 Ctrl 先松开导致无事件粘贴被丢弃；Paste/PasteImage 仍阻止重复粘贴。
- Windows 同时读取 PNG、image/png、DIBV5、DIB，后台依次验证，不能因某个格式非空就跳过有效备用格式。可用时回退系统图像读取；回退前后检查剪贴板序列号，避免粘入后来复制的图片。
- 全部候选失败时保留失败的图像数据，交给现有导入解码器报错，避免无声使用旧内部剪贴板。纯 CF_BITMAP 读取不可用、系统剪贴板被占用及宿主不触发事件仍可能需要进一步诊断。
- 新增按键顺序与重复抑制测试；位图测试覆盖 PNG 备用候选、DIB、系统回退及错误数据不被无声吞掉。
- 补丁应用和 Python 语法检查通过；Rust 测试、Windows 编译以本轮 Actions 结果为准，PixPin 实机粘贴尚未验收。

## LC-022 文档信息中的已选数量（Core v0.10.2 / 可选 Pie v0.7.2）

- “文档信息 → 对象”新增“已选物体”和“已选编组”，零选择也显示 0。
- 物体按当前所选节点计数，编组本身算一个物体，也是一个编组；不递归展开组内成员。普通物体包含路径、文字、图像等；图层与参考线不计入。
- 单独选中组内成员，只计该成员，不把其父编组当作选中；剪切组也计作编组。嵌套组整组选中时只计当前选中的组。
- 文档信息缓存包含所选节点 ID，修正仅选择变化、文档版本号未变时的陈旧统计；切换文档、范围及撤销后的查询均会更新。
- 原有文档类别统计与报告格式保留；扩展 document.info 的 selection.objects / selection.groups 供面板读取。
- 补丁：v021/patch_doc_info_selection.py。干净官方源码的补丁应用通过；新增引擎与无窗口面板回归，Windows 编译及 Rust 测试待本轮 Actions 验证。
- 配套可选 Pie 重新打包同一功能，防止安装旧可选包后丢失新增统计；默认 Core 仍不加载饼菜单。

## LC-021 动作快捷键（Core v0.10.1 / 可选 Pie v0.7.1）

- 在“动作选项”中录入、清除每个动作的快捷键；列表显示已分配按键。
- 快捷键存于动作本身：随首选项和 .vcactions 导入导出，改名/调整顺序仍绑定原动作；旧文件缺少字段时默认为无快捷键。
- 拒绝原生命令、其他动作、系统保留键冲突；可选饼菜单版也检查菜单键。后续原生快捷键重新分配后，优先保留原生命令；重复导入/复制不抢占按键。
- 文字/输入框/IME、对话框、弹出菜单、忙碌工具、录制动作和演示模式均不执行；长按重复不反复运行，动作仍通过 command.batch 撤销。
- 默认不含饼菜单的 Core 与独立可选包均集成；安装默认目录为 C:\Program Files\ArtCraft\VectorCraft。
- 源码 v021/leo_action_shortcuts.rs + patch_action_shortcuts.py；Windows 编译与测试以本次 Actions 为准，实际 GUI 待验收。


### LC-021 交付验证（2026-11-03）

- 程序源码提交：`0bbc81a591524eeeee6716e1b6eed709641bda1f`；[Windows 构建 #41585779112](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41585779112) 的两个原生 EXE 与两个安装器均成功。
- 动作专项：默认 Core 6 项 / 可选 Pie 7 项通过；覆盖配置往返、原生命令与动作冲突、改名/顺序、复制/导入、录入取消、对话框确认保护、一次按键执行、长按抑制、撤销及录制/对话框屏蔽。两种版本各 36 项原生 Actions 回归通过；可选 Pie 原有 26 项回归通过。
- 原有圆角、AI 导入、SVG/PDF 字体、ICO、选边、截图粘贴、图像导出、工作区指定回归通过。默认 Core 2 项休眠配置保留测试通过。
- 可选安装器用本次真实 EXE 验证：拒绝非配套版本、安装/卸载精确恢复 Core、保护后续更新、用户配置字节不变，全部通过。
- [Core v0.10.1 安装器](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41585779112/artifacts/13210135927)；[可选 Pie v0.7.1 安装器](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41585779112/artifacts/13209547452)。需先安装配套 Core，再按需安装 Pie。
- 使用：选中具体动作行 → 面板右上角菜单 → 动作选项 → 点击录入快捷键 → 确定。设置保存在动作数据中，.vcactions 可携带；冲突的导入绑定会清除，避免抢占现有按键。
- 自动测试包括无窗口界面事件测试；不等同于用户机器的 Windows GUI 验收。
- 后续 CI 先格式化再编译测试，避免默认 Core 重编译；相同底层的圆角和 PDF 回归只在 Core 执行，两种版本仍各自运行 UI/动作测试。当前交付 EXE 未因此改变，也未重新触发原生构建。


## v0.20 拆分开发记录（交付目标已转到 v0.21）

- 官方基线仍为 VectorCraft v0.20.0。
- 默认 Core Bridge 不包含 LC-019 饼菜单模块、菜单入口、快捷键及鼠标钩子；不是把功能隐藏或关闭。
- 其他 16 项补丁调用顺序保持不变：智能选择、整组锚点移动、彩色辅助线、文字/字体、AI 导入、截图粘贴、导出、面板及配置导入导出等继续保留。
- 历史 ICC 持久化与完整自由工作区缺口仍未宣称补齐。
- 饼菜单作为独立下载的 Leo Design Pie Menu v0.6.1 可选安装器提供；需要配套 Core Bridge v0.9.8。
- 官方 WASM API 暂不支持画布输入/面板扩展。此可选文件是同版本增强 EXE 的安装/恢复包，不是通用 WASM/DLL 插件。
- 可选包卸载后恢复同版本不含饼菜单的 Core，不回到官方裸程序；若程序已经升级则保留升级版本。
- 默认 Core 保留原菜单的原始 JSON 设置，供重装使用，不加载菜单代码或拦截输入。
- 两个构建款式均验证源代码隔离；默认版增加配置保留回归。可选包在隔离 Windows 目录测试安装、卸载、版本拒绝、后续更新保护和配置文件不变。
- 编译及测试结果以本轮 Actions 为准；用户真实 Windows GUI 手感不等同于自动测试。

## 以下为历史审计记录

**审计日期：2026-11-01（北京时间）**  
**记录范围：** GitHub `leodesign8522/leo-vectorcraft-build` 的 `main`、`leo-v018-migration`、`leo-v019-migration` 分支中的可追溯代码与构建记录。不是将全部聊天想法都视为“已开发”。  
**官方版本：** v0.19.0（2026-10-31 发布）  
**最近成功构建：** Core Bridge **v0.8.0 / 官方 v0.19.0**（迁移预览版）  
**通过的构建：** [GitHub Actions #41213956820](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41213956820)（Windows x64 release 主程序、覆盖安装器打包成功；49 项指定回归测试通过，Windows GUI 和实际安装未验收）。  
**保留的回退增强版：** v0.7.5 / 官方 v0.18.0，[构建 #41179333762](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41179333762)。  
**原有稳定基线：** `main` 的 v0.5.7 / 官方 v0.17.0（[历史构建 #40899904329](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/40899904329)）。  
**当前边界：** v0.8.0 是 v0.19.0 适配预览版，不是已完成全部实机验收的稳定版。LC-004 ICC 设置持久化、LC-012 完整自由工作区仍未确认补齐。v0.7.5 安装器不适用于 v0.19。

## 状态字段说明

- **已有代码·构建通过**：该功能的补丁在最近成功的 v0.18 workflow 中调用，整体 Windows 编译通过；不等于每项功能已有独立测试。
- **已有代码·单测通过**：除编译之外，已有该功能的 Rust 测试并在最近 workflow 中通过。
- **独立实验包**：不在 Core Bridge 内，由独立 workflow/Windows 扩展提供；不要和主程序的安装包混淆。
- **待完整验证**：已有部分实现，功能边界或实际体验仍需现场验收。
- **需求/规划**：提出过，但没有在当前成功构建中证明完成。绝不把需求算作已开发。
- **v0.19 状态**：本次独立编译和指定回归结果见 G；不得将这些结果等同于全部历史需求、真实文件和 Windows GUI 均已验收。

## A. v0.18 历史基线功能清单（v0.7.5；v0.19 验证结果见 G）

| 编号 | 模块 | 已有代码涵盖的功能 | 代码来源 | 当前证据 |
|---|---|---|---|---|
| LC-001 | Smart Select | Ctrl 直接选中组内对象、不拆组的选取/拖动逻辑 | `v018/patch_smart_interactions.py` | 已有代码·构建通过 |
| LC-002 | Group Anchor Move | A 工具配合修饰键拖动组内锚点，实现整组位置移动 | `v018/patch_smart_interactions.py` | 已有代码·构建通过 |
| LC-003 | Smart Guides | 10 类彩色辅助线、3 套预设、自定义颜色及参数 | `v018/patch_smart_guide_colors.py` | 已有代码·构建通过；吸附现场体验待验收 |
| LC-004 | ICC 工作空间 | 工作 RGB / CMYK 配置持久化（历史 v0.17 版） | `scripts/patch_color_preferences.py` | **仅 v0.17 main 确认已集成**；该脚本未出现在 v0.18 最新构建流程，v0.18 增强版不应宣称已恢复 |
| LC-005 | 颜色面板显示模式 | 固定 CMYK 显示，避免面板自动切回 RGB | `scripts/patch_color_panel_lock.py` | 已有代码·构建通过 |
| LC-006 | Illustrator 文字兼容 | AI/PDF/SVG 文字名及字体兼容补丁（非所有字体段落效果完全无损） | `scripts/patch_illustrator_text_bridge.py` | 已有代码·构建通过；精确还原待验收 |
| LC-007 | CJK PDF 字体映射 | 处理嵌入 PDF 剪贴板的 CID/CJK 编码后缀与字体族识别 | `scripts/patch_pdf_cjk_font_names.py` | 已有代码·构建通过 |
| LC-008 | 中文段落 | 文本框自动高度、避头尾集/中文标点排版相关 UI | `scripts/patch_chinese_paragraph.py` | 已有代码·构建通过 |
| LC-009 | Ctrl+Enter | 文字编辑完成/退出文字编辑 | `v018/patch_essential_features.py` | 已有代码·构建通过 |
| LC-010 | Screenshot Paste | Windows 外部截图 Ctrl+V、CF_BITMAP 回退和重复粘贴防护 | `v018/patch_screenshot_paste.py` | 已有代码·构建通过 |
| LC-011 | Workspace/Shortcut I/O | 工作区和快捷键配置各自 JSON 导入/导出、调用入口 | `v018/leo_core_bridge/workspace_io.rs` 和 `v018/patch_core_bridge.py` | 已有代码·构建通过；跨版本 schema 需验证 |
| LC-012 | Workspace Pro | 自由面板组合、分栏、停靠/拆分布局，比原生更可定制 | `scripts/patch_workspace_pro.py`（历史版）以及 v0.18 停靠补丁 | v0.18 重新集成部分逻辑；复杂布局待实机验收 |
| LC-013 | 面板尺寸系统 | 停靠面板内部调整宽/高、数值尺寸/预设、尺寸随工作区恢复、隐藏拖拽线条 | `v018/patch_panel_usability.py`, `patch_docked_panel_sizes.py`, `patch_workspace_flyout_sizes.py`, `patch_craft_ui_flyout.py` | 已有代码·构建通过；真实鼠标命中待验收 |
| LC-014 | 快捷设置面板 | 单位（常规/描边/文字）与文档颜色模式侧边栏按钮 | `v018/patch_quick_settings.py` | 已有代码·构建通过 |
| LC-015 | Image Pro Export | JPEG / PNG、DPI、ICC、画板批量、文件名、路径、设置与停靠面板 | `v018/patch_image_export.py`, `scripts/leo_image_export.rs`, `scripts/leo_image_sidebar.rs` | 已有代码·构建通过；需要逐格式输出文件验收 |
| LC-016 | Windows ICO Export | 一个 .ico 内含 16/24/32/48/64/128/256 px 多层透明图标、按画板批量导出 | `v018/leo_ico.rs`, `v018/patch_ico_export.py` | 已有代码·构建通过；Explorer 实际显示需验收 |
| LC-017 | A 工具框选边 | 小框只穿过直线/贝塞尔路径段时选中该边；保留原锚点框选 | `v018/patch_direct_marquee_edges.py` | 已有代码·构建通过；几何案例测试含于补丁，独立 workflow 未逐项运行 |
| LC-018 | Live Corner Geometry Link | 移动未圆角化的源锚点时保留 Live Corners 参数并重算圆弧 | `v018/patch_live_corner_geometry_link.py` | **已有代码·单测通过**；外观效果 Round Corners 是另一系统 |
| LC-019 | Pie Menu（饼菜单） | 多菜单、可录制键盘/修饰键+右键、8 槽位、快捷甩动、方向编辑器、圆形/列表布局、搜索原生命令、JSON 配置 | `v018/pie-menu/leo_pie_menu.rs`, `v018/patch_pie_menu.py` | **已有代码·单测通过**；真实鼠标触发、UI 手感仍需验收 |

**全局集成入口：** [`.github/workflows/build-v018-core.yml`](.github/workflows/build-v018-core.yml)。每个功能的脚本顺序、原生 Rust 编译和测试都能从这里复核。

### 审计发现的迁移缺口

- **LC-004 ICC 工作空间持久化**：`scripts/patch_color_preferences.py` 在历史主分支的 v0.17 编译流水线中有调用，然而最新 v0.18 构建流水线并**未调用它**；不能把该功能记成 v0.18 已完成。v0.19 迁移前先验证官方是否已有等价实现，若没有再移植。
- **LC-012 Workspace Pro**：v0.18 已有面板和停靠尺寸改动，但原 `scripts/patch_workspace_pro.py` 并没有直接加入 v0.18 构建流程。自由面板组合的完整旧功能要逐项核对，不得因为构建成功就当作全部恢复。

## B. 独立扩展（不属于主程序 Overlay）

| 编号 | 扩展 | 源码 | 版本与限制 |
|---|---|---|---|
| LC-020 | Explorer 缩略图预览 | `thumbnail/src/VectorCraftThumbnailProvider.cs`, `thumbnail/Install.ps1` | v0.1.0，独立 Windows .NET Shell 扩展。仅提取 .vectorcraft 文档**已有的嵌入 PNG 预览**；未内嵌预览不能自动产生真实缩略图。详见 `thumbnail/README.md`。 |

## C. 用户提出过、不能当成“已完整开发”的需求

| 编号 | 需求 | 当前状态 |
|---|---|---|
| LC-R01 | Illustrator → VectorCraft **字体、文字内容、段落、颜色无缝 100% 复制** | 目前只有部分文本兼容补丁，完整保真未验证 |
| LC-R02 | Illustrator 式任意面板拖拽重排（所有板块完全自由组合） | 工作区增强已做一部分；高度自由 DIY 不等于完整达成 |
| LC-R03 | Affinity 式任意路径 Live Corners 与外观 Round Corners 全场景联动 | 部分 Live Corners 已修；其他效果和复杂路径仍需单独测试 |
| LC-R04 | Pie Menu 中文命令关键词、真实全键位冲突接管、复杂子菜单、多菜单快速编辑 | 当前为基础实现，未完成全部高级功能 |
| LC-R05 | 完整的 Windows GUI 自动化验收系统（真实按键、鼠标拖动、截图对比） | 尚未落地；自动化仅覆盖部分 Rust 单元测试 |
| LC-R06 | Illustrator `.aip` 图像专业导出插件的完整源码及 ICO 升级 | 这**不是** VectorCraft Core Bridge 的同一工程；目前仓库不含完整可编辑的 Illustrator `.aip` 工程 |
| LC-R07 | 官方 VectorCraft v0.19.0 Leo Design 增强兼容版 | 已生成 v0.8.0 适配预览版，Windows release 编译及 49 项指定回归通过；GUI 与实际安装验收待完成 |

## D. 每次官方更新的标准流程

1. 确认官方发行版（例如 `storytold/vectorcraft@v0.19.0`）及上游更新日志。
2. **保留旧基线**：`main` v0.5.7/v0.17、`leo-v018-migration` v0.7.5/v0.18 不覆盖；从 v0.18 的成功提交开独立 `leo-v019-migration` 分支。
3. 对照 **LC-001 ～ LC-020** 逐项移植，记录：“已移植 / 已由官方支持无需补丁 / 冲突待修 / 暂不支持”。
4. 先在 GitHub Actions 编译官方 v0.19.0 原版，再跑所有移植补丁、Clippy/编译、单元/集成测试；不得直接把 v0.18 的 EXE 安装器用于 v0.19。
5. 测试重点：原生 egui/停靠组件、A 工具/圆角、截图剪贴板、文字 PDF 导入、颜色 ICC、ICO 导出、饼菜单快捷键。
6. 输出**两个清晰命名**的 Artifact：独立 portable 版、版本检查的 Overlay Installer；使用者只需安装其中之一。旧主程序和现有工作配置不自动清除。
7. Windows GUI 实测通过后才更新“可用/稳定”标记；旧版 artifact 不要在新版本验收之前删除。

## E. 个人数据 / 工作区保存规则

- **仅导出“工作区 JSON + 快捷键 JSON”不能完整备份所有偏好**：ICC 文件、色彩设置、饼菜单配置、用户扩展等可能在独立首选项或资源目录里。
- 可以保留“日常只导出两份 JSON”的快捷流程，但升级前必须区分用户配置与程序安装，至少保证其他偏好数据仍存于原有持久目录、不被覆盖安装器误删。
- 可以重新下载官方程序，但不代表一份被覆盖或丢失的用户配置也能从官网恢复。
- 文件位置和格式随官方版本可能发生变化；升级前对照 schema 验证。

## F. 维护与版本纪律

- 功能总账是 **GitHub 中的文本记录**，不是 ChatGPT 隐藏记忆；任何新功能必须追加功能编号、源码路径、状态、测试记录、目标官方版本。
- 迁移前先对比上游是否已原生实现相同能力，避免重复维护冲突补丁。
- 开发过程的绿色构建只是“编译通过”，不可自动宣称“安装通过 / 功能全部可用”。
- 每个官方版本独立标记基线/分支；持续更新该清单，而不是在旧记录上直接覆盖历史。

## G. v0.19 / Core Bridge v0.8.0 迁移结果

**验证时间：2026-11-01（北京时间）**  
**代码提交：** `53efcc6754c4985a4fd68f71c513c31e46a40cca`  
**官方基线：** `v0.19.0 / bcebabe3f8c23a376ac68e9e6726590af278f7bb`  
**构建证据：** [#41213956820](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41213956820)，主程序和覆盖安装器两个 job 均成功。

### 下载

- [v0.8.0 覆盖安装器 ZIP（适用于官方 v0.19.0）](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41213956820/artifacts/13047898842)
- [v0.8.0 独立便携版 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/41213956820/artifacts/13048448364)

覆盖安装器仅替换 `vectorcraft.exe`；只接受 v0.19.0 / 0.19.0.0，读取不到版本或版本不匹配时停止。安装器已完成编译打包，尚未在用户机器实际执行验收。便携版解压到空目录运行，配置目录与已安装版可能不同，不等于自动搬迁全部设置。

### 逐项状态

- LC-001/002/003/005～011/013～016/018/019：v019 独立补丁应用和 Windows release 主程序编译通过。源码在 `v019/`，共用模块仍取自 `scripts/`。各项的独立回归范围见下表；未列入的交互场景不应标记为实机通过。
- LC-017：使用官方 v0.19 原生框选路径段，保留官方新命中范围及段选择逻辑；旧补丁不再重复注入。
- LC-004：历史 ICC 工作空间持久化补丁尚未恢复，官方 v0.19 是否已有等效持久化尚未验收，不能标记为已补齐。
- LC-012：现有停靠尺寸和面板增强已迁移；完整任意面板自由布局仍待核验。
- LC-020：独立 Explorer 扩展未修改，v0.19 文档缩略图兼容性仍待实测。

### 实际运行的指定回归（49 项）

| 范围 | 通过数 | 验证内容 |
|---|---:|---|
| Live Corners | 2 | 源锚点移动、圆角重新计算、撤销 |
| Pie Menu | 9 | 八方向映射、远距离甩动、精确快捷键、Shift+D、普通右键、旧配置及 JSON 八槽位保留 |
| ICO | 3 | 文件头与偏移、尺寸校验、多尺寸渲染、非正方形画板透明边距 |
| 官方 marquee | 27 | 框选路径段、锚点、参考线、画板和相关原生工具回归 |
| Screenshot Paste | 3 | Ctrl+V 图片回退、文字粘贴去重、普通 V 不误触 |
| JPEG/PNG 参数 | 2 | CMYK/DPI、透明 PNG、批量画板范围 |
| 工作区 JSON | 3 | 格式/版本检查、交换往返、空名称拒绝 |
| **合计** | **49** | **0 失败；不是完整全量测试套件** |

### 本次适配修复

- 适配官方 Selection / Direct Selection 方法拆分，保留 `selected_source_hit` 的命中范围。
- Live Corners 保留上游 `outermost` 过滤，避免父子对象重复变换。
- Pie Menu 适配新版 `canvas_input` / `canvas_cursor`，保留原快捷键和配置格式。
- ICO 格式注册保留官方新加入的 PCX 导入项。
- craft-ui 按 Cargo.lock 的准确提交选择补丁目标，避免改错另一版本缓存。
- 修复 Windows 截图模块的 `MAX_SIDE` 常量作用域。
- 主程序保持 release 优化；单元回归改用常规 test 配置，减少重复优化和链接耗时。

### 仍需验收

真实 Windows 鼠标/键盘手感、高 DPI、多显示器、实际安装、复杂工作区、旧快捷键/饼菜单/ICC 资源、真实 AI/PDF 文件与字体表现均尚未完成用户实机验收。不要把 49 项指定回归通过解释成所有历史功能和需求均已完整实现。

## H. v0.20 / Core Bridge v0.9.0 迁移（2026-11-02）

- 基于最新已通过 Windows 构建的 v019 中文饼菜单提交 `834326ebc4b6626058c1851b84a2fd92cd89180f`。
- 独立 `v020/` 与 `build-v020-core.yml`；固定官方 `5628de96f866185d8eb289b167b89691f213c0c3`。
- 所有补丁已在干净 v0.20.0 上顺序应用，源码空白检查通过。
- PDF CJK 后缀处理适配官方新字体匹配，保留连字符字体族识别。
- ICO 注册保留 v0.20 新增 JPEG 2000，不回退官方格式列表。
- LC-017 保留官方原生选边；LC-019 保留中文列表、双语搜索及三套预设。
- Windows 编译与指定回归待本次 Actions；GUI、实际安装和用户配置迁移待验收。
- LC-004 / LC-012 仍未宣称完成。详情见 `v020/MIGRATION_CHECKLIST.md`。

## I. 饼菜单便捷配置状态（2026-11-02）

- v0.9.2：中文搜索、同义词、预设、原生配置文件导入导出已集成，Windows 构建及指定测试成功（#41390119248）。
- v0.9.3：根据用户实机反馈，删除未能使用的原生菜单右键添加入口；不宣称该功能可用。
- 保留画布饼菜单触发、现有槽位、命令参数及文件备份恢复兼容。

### 2026-11-02 Core Bridge v0.9.6

- 新增文件菜单 AI 原始编辑数据优先入口及实际导入来源/警告报告；文字溢出提示，保留原 AI 不覆盖。
- 饼菜单增加最多 8 层子菜单、创建/挂载/返回/删除引用维护及 JSON 备份兼容。
- 旧快捷键、画布右键组合和叶子命令参数执行保持兼容；子菜单进入后采用单击选择。
- 当前为源码开发验证；Windows 构建/自动回归以对应 Actions 为准，实机交互未验收。
- 未包含 Illustrator 端脚本或面板；缺失字体、复杂外观和不支持的原始数据仍会降级并报告，不能称为无损桥接。




## v0.10.6 — LC-027 / LC-028

- A 工具选中单条或多条线段，Ctrl+C / Ctrl+X 提取所选几何，不再复制整个闭合路径。
- 保留连接关系、闭合路径跨起点的相邻线段、独立子路径、曲线手柄和对象样式；复制不改变源对象。
- 部分路径提取后解除 live shape 生成器，避免编辑时恢复未复制的边；整对象复制继续使用原生路径。
- Shift 水平/垂直拖点或移动部分选中线段时，Smart Guides 可吸附可见参考线，独立于 Snap to Point；锁定参考线仍可吸附，隐藏后不参与。
- 延长线沿用青色虚线预览、琥珀色虚线与命中点；按 Shift 锁定方向时不再显示另一方向的延长线预览。
- 验证门槛：补丁锚点精确匹配；Rust 几何、复制/剪切/粘贴/SVG/撤销与参考线方向回归；两种程序编译；安装器配对校验。
- Windows 上 PixPin、复杂文件与实际操作手感仍需实机验收。

