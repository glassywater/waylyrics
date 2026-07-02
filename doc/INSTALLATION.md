# Waylyrics 安装指南

## AppImage 安装（推荐）

AppImage 是最简单的安装方式，无需安装额外依赖。

### 下载

从 Release 页面下载 `waylyrics-{version}-x86_64.AppImage`。

### 运行

```bash
chmod +x waylyrics-*.AppImage
./waylyrics-*.AppImage
```

### 自行构建

#### 依赖

- Rust 工具链（rustc, cargo）
- GTK4 开发库
- appimagetool

#### 构建步骤

```bash
# 安装 appimagetool（如果未安装）
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /usr/local/bin/appimagetool
chmod +x /usr/local/bin/appimagetool

# 构建
cd waylyrics
make appimage
```

#### 构建产物

- `AppDir/` - AppDir 目录结构
- `waylyrics-{version}-x86_64.AppImage` - 可执行的 AppImage 文件

## 从源码构建

### 依赖

- Rust 1.80.0+
- GTK4
- gettext（可选，用于国际化）
- dbus（可选，用于 MPRIS）
- openssl

### Fedora

```bash
sudo dnf install gtk4-devel gettext-devel dbus-devel openssl-devel
```

### Ubuntu/Debian

```bash
sudo apt install libgtk-4-dev gettext libdbus-1-dev libssl-dev
```

### 构建

```bash
cd waylyrics
cargo build --release
```

### 运行

```bash
./target/release/waylyrics
```

## 配置

配置文件位于 `~/.config/waylyrics/config.toml`。

### 字体配置

在配置文件中添加以下选项：

```toml
# 字体家族（支持系统已安装的任何字体）
font-family = "Noto Sans CJK SC"  # 中文
# font-family = "Microsoft YaHei"  # Windows 中文字体
# font-family = "PingFang SC"      # macOS 中文字体
# font-family = "Sans"             # 默认
```

主题文件位于 `~/.local/share/waylyrics/_themes/`。

### 字体大小配置

编辑主题 CSS 文件修改字体大小：

```css
label#above {
  font-size: 28px;
}

label#below {
  font-size: 24px;
}
```

## 故障排除

### 主题未找到

确保主题文件存在于 `~/.local/share/waylyrics/_themes/` 目录。

### GSettings schema 错误

如果从源码构建，可能需要安装 GSettings schema：

```bash
sudo cp metainfo/io.github.waylyrics.Waylyrics.gschema.xml /usr/share/glib-2.0/schemas/
sudo glib-compile-schemas /usr/share/glib-2.0/schemas/
```

### Locale 翻译未找到

AppImage 会自动编译 locale 文件。如果翻译不生效，检查 `~/.local/share/waylyrics/` 目录权限。
