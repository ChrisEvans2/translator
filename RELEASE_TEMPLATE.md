# Release Template

复制以下模板，替换版本号和内容：

---

# v{VERSION} Release Notes

## 新功能

- 功能 1
- 功能 2

## 修复

- 修复 1
- 修复 2

## 优化

- 优化 1
- 优化 2

## 安装说明

### Windows
下载对应架构的安装程序：
- `translate_app_{VERSION}_x64-setup.exe` — NSIS 安装程序（推荐）
- `translate_app_{VERSION}_x64_en-US.msi` — MSI 安装程序

运行安装程序，按照提示完成安装。

### 从源码构建
```bash
git clone https://github.com/ChrisEvans2/translator.git
cd translator
pnpm install
pnpm tauri build
```

## 系统要求

- Windows 10/11（需要 WebView2）
- 从源码构建需要：Node.js 18+、pnpm、Rust 1.70+

## 完整更新日志

详见 [CHANGELOG.md](https://github.com/ChrisEvans2/translator/blob/main/CHANGELOG.md)
