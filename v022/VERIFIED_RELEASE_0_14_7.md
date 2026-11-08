# Leo Design Core Bridge 0.14.7 验证记录

VectorCraft 0.22.0 / Windows x64；保留 Leo Snapping Engine 1.7。

## 修复贴合字形后的对象对齐

0.14.6 的“贴合字形选框”修改了选择/变换控件的边界，但对象对齐仍通过另一个 Measure 读取字体度量框或段落框。因此，水平、垂直及同时居中可能与可见字形不一致。这是上一版漏接的调用链。

0.14.7 把已开启 `leoGlyphBounds` 的文字和普通组内文字接入 Align/Distribute 共用测量。对象自身、选择范围、关键对象及画板参考都使用与贴合选框一致的边界。原有全局 Align to Glyph Bounds 选项仍适用于未开启此开关的文字；关闭贴合、无字形或不支持的变形继续走原有测量。

- 开启“贴合字形选框”后，水平居中、垂直居中、水平和垂直同时居中按实际字形轮廓计算。其他对象边缘对齐及分布也共用相同测量入口。
- 以矩形为关键对象时，矩形保持不动，文字只作整体平移，字号、字形、旋转、段落框与换行不变。
- 保留 0.14.6 的选框、编辑、保存重开与撤销重做行为。字符面板开关的提示同步说明对齐规则。

用法：选中文字，开启 **字符 → 贴合字形选框**（或文字菜单同名命令）。选文字和矩形，再单击矩形设为关键对象，然后点击居中按钮。以画板或选择范围为参考时也支持字形测量。

## 自动化回归

- 官方源码：`654749c16eb20b6d3182a96defb66e11d1a9649a`。
- 构建源码：`887dd9380a5c9fc3b75cd7f965bba0a6148ec5ea`。
- [专项回归](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42465551858)：成功；11 项文字回归通过。新增覆盖点/多行文字 × 普通/预览边界 × 三种居中、矩形关键对象不动、旋转文字、关键文字、画板与选择范围、关闭开关及撤销重做。
- 保留 53 项吸附引擎、17 项对象移动、77 项 Direct Selection、83 项 Selection、14 项原生变换工具筛选（筛选有重叠，不相加）。
- [完整 Windows 构建、保留功能回归和安装校验](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42465551859)：成功。

## 实际程序验证与旧版反例

使用隔离便携程序和合成 B5/多行文字，直接运行图片中按钮注册的 UI 命令 `align.horizontalVerticalCenter`，同时检查两个单轴按钮。独立测量使用临时 Create Outlines 后的几何边界，并立即撤销恢复可编辑文字；没有用新对齐测量函数自身充当正确答案。

- 0.14.6：点文字和段落文字 × 三种居中，6 种均检出中心偏差。
- 0.14.7：相同 6 种通过，字形中心误差小于 `1e-7` 文档单位；矩形不动，文字数据仅改变变换的平移分量，撤销/重做通过。
- 保留五种文字渲染不变、保存重开及编辑检查，以及 R 旋转与 V 四边缩放实际程序回归，均通过。
- 已查看实际 B5/矩形居中截图。

## 安装包与摘要

请从 **Leo Design VectorCraft 0.22 Bridge 0.14.7** 新快捷方式启动。安装目录独立，快捷方式带 `--new-instance`，避免进入旧版窗口。

- [安装器 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42465551859/artifacts/13611954216)。
- [便携 ZIP](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42465551859/artifacts/13611974218)。
- 主程序 SHA-256：`97ad58371a76e63024ac31b6a8f7ec9a6974fcb5723b237b4241b73a8a7b1d3e`。
- 安装器 SHA-256：`0a69785edf278741b61cbd383445b0cafb7423f41d35bc98ba6b5f8d919576db`。
- 已核对 GitHub artifact digest、BUILD_COMMIT、UPSTREAM_COMMIT、Windows x64 PE、`--version` 和随包 SHA256。
