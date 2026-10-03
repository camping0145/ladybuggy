# 🐞 Ladybuggy Terminal (V2)

A fast, lightweight, and minimalistic Wayland native terminal emulator based on the **Foot** engine. Built in optimized **C11** for extreme rendering speeds and low-latency input tracking on **Omarchy**.

## ✨ Core Features
* **Wayland Native:** High-performance, damage-tracked presentation.
* **Server/Daemon Mode:** Host multiple terminal windows in a single background process.
* **Strict Prompt Locking:** Preserves prompt arrow layout (`❯`).
* **Keyboard-Driven URL Mode:** Rapid keyboard navigation for URLs.
* **Sixel Graphics Support:** Native embedded imagery handling.

## 🚀 Getting Started

Install prerequisites (e.g., Arch/Omarchy):
```bash
sudo pacman -S base-devel pkg-config meson ninja pixman libwayland-client wayland-protocols tllist fcft libxkbcommon fontconfig libseccomp
```

Clone, build, and compile:
```bash
git clone https://github.com
cd ladybuggy
meson setup --buildtype=release build
ninja -C build
```

## 📦 Global Installation
```bash
sudo cp build/ladybuggy /usr/local/bin/
sudo chmod +x /usr/local/bin/ladybuggy
sudo mkdir -p /usr/local/share/icons/hicolor/256x256/apps/
sudo cp icons/ladybuggy.png /usr/local/share/icons/hicolor/256x256/apps/ladybuggy.png
sudo cp ladybuggy.desktop /usr/local/share/applications/
sudo update-desktop-database /usr/local/share/applications/ 2>/dev/null
```

## 🛠️ Configuration
Config file location: `~/.config/ladybuggy/ladybuggy.ini`.

## 📄 License
MIT License.
