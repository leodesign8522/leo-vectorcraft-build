# Leo Design × VectorCraft — 功能总账（长期维护）

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
