# MixTex-rs GUI

[![License](https://img.shields.io/badge/license-GPLv3-blue)](https://www.gnu.org/licenses/gpl-3.0)
[![Tauri](https://img.shields.io/badge/Tauri-2.1.0-red?logo=tauri)]()
[![Vue.js](https://img.shields.io/badge/vue.js-v3-green?logo=vue.js)](https://github.com/vuejs/vue-next)
[![Rust](https://img.shields.io/badge/-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
![Platform](https://img.shields.io/badge/platform-Windows%20|%20Mac%20|%20Linux-orange)


> [!info]
> v0.5.0 更新：
> - 使用Tauri v2的channel和stream，加速推理和截图
> - 支持多推理后端
> - 改进UI，使用[LatexLive](https://github.com/MosRat/LaTeXLive)的开源版本作为编辑器

A GUI implement of [MixTex](https://github.com/RQLuo/MixTeX-Latex-OCR/tree/MixTeX-v1.1.2), use [Rust](https://www.rust-lang.org/) + [Vue](https://github.com/vuejs/) + [Tauri](https://github.com/tauri-apps/tauri)


##  Setup and Usage

- Download installer for your system from [Release](https://github.com/MosRat/MixTex-rs-GUI/releases/latest), or single exe (Windows only)
- Use `Shift + X`  to screenshot latex. (can be changed in future version)
- Or drag png/jp(e)g image into app
- Or click `Select` to choose one from filesystem.

![result_3dview2.gif](docs%2Fgif%2Fresult_3dview2.gif)

## Exit
Default behavior of click close button is minimize.The shortcut call will restore it.To close it, find it in taskbar and right click.
![img.png](docs/img/img.png)
## Platform

**No tests on Linux and Mac yet**

I only have Windows PC. Release of other systems is auto created by github actions.

## Develop

- Download model and lib from [Deps](https://github.com/MosRat/MixTex-rs-GUI/releases/tag/deps)
- ``` git clone ``` this repo and copy model to {repo}/models, lib to {repo}/lib.
- Prepare rust, node and tauri.
- set env ``` ORT_LIB_LOCATION=path/to/your/lib/folder  ```
- ```pnpm i ```
- ``` cargo tuari dev --release ``` (libs are built in release mode, debug build will lead to linker error on Windows msvc)

