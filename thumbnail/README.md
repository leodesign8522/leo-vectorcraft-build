# Leo Design Thumbnail Preview v0.1.0 (Windows 11 x64)

Independent Windows File Explorer thumbnail handler for native .vectorcraft documents.

## Install (once)

1. Download the ZIP from the **Thumbnail Preview Windows** GitHub Actions artifact.
2. Extract the whole ZIP to any folder.
3. Double-click **Install.cmd** and approve the Windows administrator prompt.
4. Open File Explorer and choose **Large icons / Extra large icons**. Restart Explorer if necessary.

No Rust, AutoHotkey, Visual Studio or changes to your VectorCraft EXE are needed.

## IMPORTANT: What is supported

VectorCraft's official native format can store an **optional** PNG preview of the FIRST artboard (max 256 px). This independent handler reads that existing image from uncompressed JSON or gzip-compressed .vectorcraft files.

**Files without an embedded preview cannot show actual artwork through this v0.1.0 handler.** VectorCraft's own native file format defines Preview as an optional save option, default **false**. In VectorCraft, use **Save As** and enable **Preview**, then save a COPY. Simply installing the handler cannot recover an image that was never saved in the file. Existing files are not rewritten by this extension.

This release does not render the VectorCraft document or implement a separate Explorer Preview Pane handler. Windows may scale the embedded 256px preview when displaying extra-large icons.

## Uninstall

Double-click **Uninstall.cmd**, approve Windows administrator prompt, then restart File Explorer if the thumbnail cache persists. The installer backs up and restores any previous .vectorcraft thumbnail handler registration.

## Safety and compatibility

- 64-bit Windows 10/11 with the built-in .NET Framework 4.x.
- Installs a COM DLL under Program Files and registers it as the Windows x64 thumbnail provider. It does **not** replace or modify the VectorCraft application.
- Reads the beginning of .vectorcraft files only, with a 16 MB decode cap; it does not execute document contents.
- Gzip and plain native preview PNGs supported. Missing preview falls back to the Windows generic file icon.
- This is an experimental **unsigned** self-built extension. Initial cloud build/smoke tests do not prove that Explorer registration and rendering work on your particular Windows desktop.

Upstream reference: https://github.com/storytold/vectorcraft/blob/v0.17.0/crates/format/src/lib.rs
