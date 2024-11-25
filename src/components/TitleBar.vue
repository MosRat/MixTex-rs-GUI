<script setup lang="ts">
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
const appWindow = getCurrentWebviewWindow()
</script>

<template>
<div data-tauri-drag-region class="titlebar">
  <div data-tauri-drag-region class="title">
    <img class="icon" src="/mixtex.png">
    MixTex-rs-gui
  </div>
  <div class="titlebar-button" id="titlebar-minimize" @click="appWindow.minimize">
    <img
      src="/mdi_window-minimize.svg"
      alt="minimize"
    />
  </div>
  <div class="titlebar-button" id="titlebar-maximize" @click="appWindow.toggleMaximize">
    <img
      src="/mdi_window-maximize.svg"
      alt="maximize"
    />
  </div>
  <div class="titlebar-button" id="titlebar-close">
    <img src="/mdi_close.svg" alt="close" @click="appWindow.minimize" />
  </div>
</div>

</template>

<style scoped>
.title{
  flex-grow: 1;
  display: flex;
  align-items: center; /* 垂直居中对齐 */
  font-size: small;
}
.icon{
  width: 36px;
  margin-inline: 1.25vw;
}

.titlebar {
  height: 5vh;
  background: transparent;
  user-select: none;
  display: flex;
  justify-content: flex-end;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;

  padding: 0;
  margin: 0;
}
.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: min(5vw,3em);
  height: 5vh;
}
.titlebar-button:hover {
  background: rgba(91, 91, 91, 0.5);
}

#titlebar-close:hover {
  background: rgba(255, 12, 12, 0.5);
}
</style>