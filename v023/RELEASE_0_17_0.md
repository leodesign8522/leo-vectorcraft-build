# Leo Design Core Bridge 0.17.0 / VectorCraft 0.23.0

## 本次升级

- 以官方 VectorCraft 0.23.0 为基础，保留 Core Bridge 0.16.2 的现有增强。
- 删除 Core Bridge 自带的 Ctrl＋Enter 文字编辑处理，使用官方实现。
- 保留统一吸附、真实/参考线交点、8px 捕获/12.8px 释放、Shift/Alt 兼容、编组整体移动、延长线及旋转吸附。
- 保留文字选框/高度模式、吸管、动作快捷键、工作区及面板、对象统计、图像/ICO 导出、中文字体兼容和 AI 双向发送。
- 饼菜单与自定义截图粘贴补丁继续停用。

## 验证

- [干净官方源码迁移与 Rust 语法检查](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42825601437)通过。
- [Windows 回归、正式编译与安装校验](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42825601622)通过。
- 官方 Ctrl＋Enter 工具、UI、输入法组合、重复按键及自定义快捷键优先级测试通过。
- 工作流配置的 AI 联动/导出、吸附、文字、动作、对象统计、剪贴板、圆角、AI/PDF/SVG 字体、图像/ICO 与工作区回归均通过。
- 安装到隔离目录后，EXE 哈希与测试构建一致，版本识别检查通过。
- 未在用户桌面重新进行鼠标手感验收，也未重新运行 Illustrator COM 往返；已有 AI 桥接支持范围及限制保持。

## 下载与启动

[下载单一完整安装包](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42825601622/artifacts/13773096333)。解压后运行 EXE。
启动 **Leo Design VectorCraft 0.23 Bridge 0.17.0**。
安装包已包含吸附功能和现有 AI 发送脚本，无需另外安装吸附插件。

源码提交：`3f498d77caa67d072dd0904f8667a2623d67c246`。
官方源码：`5d0e892e2419f63bc0ea3c743ddff2e81e15fd3a`。
安装包压缩文件 SHA-256：`sha256:5a0a09d0b04a90221f21918a109025aaa00650a7389fd9d07c852300fb8f3919`。
