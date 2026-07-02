
- [下载预编译二进制](#下载预编译二进制)
- [通过包管理器安装](#通过包管理器安装)
  - [Fedora Copr](#fedora-copr)
  - [archlinuxcn](#archlinuxcn)
  - [Flatpak](#flatpak)
  - [Spark Store (Ubuntu 22.04 LTS)](#spark-store-ubuntu-2204-lts)
- [安装构建依赖](#安装构建依赖)
  - [Debian-based](#debian-based)
  - [Arch-based](#arch-based)
  - [其他RPM系发行版：](#其他rpm系发行版)
  - [Windows](#windows)
- [编译](#编译)
  - [使用 stable 工具链](#使用-stable-工具链)
  - [使用 nightly 工具链](#使用-nightly-工具链)
  - [本地安装](#本地安装)
    - [编译Schema](#编译schema)
    - [本地化文件](#本地化文件)
    - [Desktop 文件](#desktop-文件)
  - [打包](#打包)
  - [AppImage 打包（推荐）](#appimage-打包推荐)

可以在 [Actions](https://github.com/waylyrics/waylyrics/actions/workflows/smoketest.yml) 下载发布

# 下载预编译二进制

[builds]: https://github.com/waylyrics/waylyrics/actions/workflows/test.yml

我们在 [github action][builds] 提供下载。

这些构建将 `WAYLYRICS_THEME_PRESETS_DIR` 设置为 `/usr/share/waylyrics/themes`，

你可以把主题放在 `${XDG_DATA_HOME}/_themes/`，waylyrics 会先尝试从这里加载。

# 通过包管理器安装

[![Packaging status](https://repology.org/badge/vertical-allrepos/waylyrics.svg)](https://repology.org/project/waylyrics/versions)

## Fedora Copr

Fedora 用户可以使用 [yohane-shiro/waylyrics](https://copr.fedorainfracloud.org/coprs/yohane-shiro/waylyrics)

## archlinuxcn

Arch Linux 用户可以使用 [archlinuxcn](https://github.com/archlinuxcn/repo) 源安装

## Flatpak

<a href='https://flathub.org/apps/io.github.waylyrics.Waylyrics'>
    <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=zh-Hans'/>
</a>

## Spark Store (Ubuntu 22.04 LTS)

<a href='https://www.spark-app.store/'>
    <img width='120' alt='去星火商店下载' src='https://gitee.com/spark-store-project/spark-store/raw/dev/src/assets/tags/community.png'/>
</a>

Ubuntu 22.04 用户可以去星火商店安装，其他版本没有测试。
```shell
sudo aptss install waylyrics
```

# 安装构建依赖

## Debian-based

```bash
sudo apt-get install libssl-dev libgtk-4-dev libdbus-1-dev libmimalloc-dev gettext
```

## Arch-based

```bash
paru -S gtk4 libxcb mimalloc
```

## 其他RPM系发行版：

请安装如下依赖：

```
cargo libgraphene-devel gtk4-devel openssl-devel dbus-1-devel mimalloc-devel pango-devel gettext
```

## Windows

请查阅 [gtk book](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html#install-gtk-4) 安装 gtk4

如果要使用 MSVC 请启用 `--no-default-features` ，gettext-rs 不支持 Windows MSVC 编译

对于 `opencc` ，则需要你复制他们的预构建发布至 `%systemdrive%\gtk-build\gtk\x64\release` 。

# 编译

```bash
export WAYLYRICS_THEME_PRESETS_DIR=/usr/share/waylyrics/themes
```

waylyrics 会从该位置加载主题，除非被 `${XDG_DATA_HOME}/_themes/<name>.css` 覆盖

如果编译时没有设置这个环境变量，waylyrics将只能加载用户主题。

## 使用 stable 工具链

```bash
cargo build --release --locked --target-dir target
```

## 使用 nightly 工具链

```bash
cargo +nightly build --release --locked --target-dir target
```

生成的二进制会被放在 `./target/release/`

## 本地安装

### 编译Schema

```bash
install -Dm644 metainfo/io.github.waylyrics.Waylyrics.gschema.xml -t ~/.local/share/glib-2.0/schemas/
glib-compile-schemas ~/.local/share/glib-2.0/schemas/
```

### 本地化文件

```bash
cd locales
for po in $(find . -type f -name '*.po')
do
    mkdir -p ~/.local/share/locale/${po#/*}
    msgfmt -o ~/.local/share/locale/${po%.po}.mo ${po}
done
```

### Desktop 文件

```bash
install -Dm644 metainfo/io.github.waylyrics.Waylyrics.desktop -t ~/.local/share/applications
```

## 打包

打包脚本样例：

```bash
install -Dm644 ./metainfo/io.github.waylyrics.Waylyrics.gschema.xml -t /usr/share/glib-2.0/schemas/
install -Dm644 ./metainfo/"io.github.waylyrics.Waylyrics.desktop" -t /usr/share/applications/
install -dm755 /usr/share/waylyrics/themes
cp -r ./themes/* /usr/share/waylyrics/themes/
cp -r ./res/icons/* /usr/share/icons/

cd locales
for po in $(find . -type f -name '*.po')
do
    mkdir -p /usr/share/locale/${po#/*}
    msgfmt -o /usr/share/locale/${po%.po}.mo ${po}
done
```

## AppImage 打包（推荐）

AppImage 是最简单的分发方式，无需安装额外依赖，用户可直接运行。

### 依赖

- Rust 工具链（rustc, cargo）
- GTK4 开发库（见上方安装构建依赖）
- appimagetool
- msgfmt（可选，用于编译 locale 文件）
- glib-compile-schemas（可选，用于编译 GSettings schema）

### 安装 appimagetool

```bash
# 下载 appimagetool
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /usr/local/bin/appimagetool
chmod +x /usr/local/bin/appimagetool
```

### 构建步骤

```bash
cd waylyrics

# 方式一：使用 Makefile（推荐）
make appimage

# 方式二：手动构建
./build-appimage.sh
ARCH=x86_64 appimagetool AppDir waylyrics-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')-x86_64.AppImage
```

### 构建产物

- `AppDir/` - AppDir 目录结构
- `waylyrics-{version}-x86_64.AppImage` - 可执行的 AppImage 文件

### 运行 AppImage

```bash
chmod +x waylyrics-*.AppImage
./waylyrics-*.AppImage
```

### AppImage 包含的内容

- 编译后的二进制文件
- GSettings schema（已编译）
- 主题文件（CSS）
- Locale 翻译文件（已编译）
- Desktop 文件
- 图标文件
- Metainfo 文件

### 配置

AppImage 首次运行会自动复制主题到用户目录 `~/.local/share/waylyrics/_themes/`。

字体配置请编辑 `~/.config/waylyrics/config.toml`：

```toml
font-family = "Noto Sans CJK SC"
```
