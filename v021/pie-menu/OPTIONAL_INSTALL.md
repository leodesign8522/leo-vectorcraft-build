# Leo Design Pie Menu v0.7.3 — 可选安装包

适配：Leo Design Core Bridge v0.10.3 / VectorCraft v0.21.0。

- 默认 Core Bridge 已移除饼菜单界面、模块、快捷键拦截和鼠标拦截。
- 需要时，关闭 VectorCraft，运行单独的 Pie Menu 安装器，选择已安装 Core Bridge v0.10.3 的目录。
- 不想用时，在 Windows“已安装的应用”中卸载 Leo Design Pie Menu；恢复同一版本的不含饼菜单的 Core Bridge，其他增强功能保留。
- 卸载不会删除 VectorCraft 用户设置。默认 Core 仅保留原饼菜单的原始配置数据，供以后重装使用，不运行菜单或拦截输入。
- 该文件是单独的可选安装器。当前官方 WASM 接口没有画布输入或面板扩展接口，因此它通过替换同版本增强 EXE 实现；不是可直接放入 Plug-ins 的 WASM/DLL 插件，也不是跨版本通用插件。
- 安装校验精确的 Core / Pie EXE SHA-256；卸载只恢复与本安装包配套的 Core。如果主程序后来升级，卸载仅移除本可选包，不覆盖更新后的程序。
- 本包沿用已有饼菜单功能，没有宣称修复全部实际操作手感。

Core v0.10.3 / Pie v0.7.3 增加每个动作的快捷键：选中具体动作 → 面板菜单 → 动作选项 → 点击录入快捷键 → 确定。快捷键保存在动作与动作集文件中。
默认安装目录：`C:\Program Files\ArtCraft\VectorCraft`。
