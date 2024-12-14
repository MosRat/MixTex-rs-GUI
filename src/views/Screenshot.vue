<!--
  - *Copyright (c) 2024. MosRat
  - All rights reserved.
  -
  - Project: mixtex-rs-gui
  - File Name: Screenshot.vue
  - Author: MosRat (work@whl.moe)
  - Description:
  -->

<script setup lang="ts">
import {onMounted, ref} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import {info, warn} from "@tauri-apps/plugin-log";
import {getCurrentWebviewWindow} from "@tauri-apps/api/webviewWindow";
import {currentMonitor, Monitor, UserAttentionType} from '@tauri-apps/api/window';

info(">>>>>>>>>>>>>Vue setup>>>>>>>>>>>>")

const appWindow = getCurrentWebviewWindow()


// const imgUrl = ref('');
const imgRef = ref<HTMLCanvasElement | null>(null);
const isMoved = ref(false);
const isDown = ref(false);
const mouseDownX = ref(0);
const mouseDownY = ref(0);
const mouseMoveX = ref(0);
const mouseMoveY = ref(0);
const screen = {
  width: window.innerWidth,
  height: window.innerHeight
};

info(`${screen}`)


onMounted(async () => {
  info(">>>>>>>>>>>>>Screenshot window mount!>>>>>>>>>>>>")

  await appWindow.listen<void>("activate", async () => {
    invoke<ArrayBuffer>('screenshot', {}).then(
        async (uint8Array: ArrayBuffer) => {

          const monitor = await currentMonitor() as Monitor
          const rect = monitor.size
          const clampedArray = new Uint8ClampedArray(uint8Array);

          // 获取 Canvas 和上下文
          const canvas = document.getElementById('canvas') as HTMLCanvasElement;
          const context = canvas.getContext('2d') as CanvasRenderingContext2D;

          // 获取 Canvas 的宽度和高度
          const width = rect.width;
          const height = rect.height;

          canvas.width = width
          canvas.height = height

          // 创建 ImageData 对象
          // const imageData = context.createImageData(width, height);
          const imageData = new ImageData(clampedArray, width, height);


          // 将 ImageData 绘制到 Canvas
          context.putImageData(imageData, 0, 0);


          // // 创建 Blob 对象
          // const blob = new Blob([uint8Array], {type: 'image/png'});
          //
          // // 创建一个 URL 对象
          // imgUrl.value = URL.createObjectURL(blob)

          await onImageLoad()

        }
    )

    try {
      await appWindow.once("success_save", async () => {
        await info("once success from rust!")
        await appWindow.hide();
      })

    } catch (error) {
      await warn(`Error in screenshot process:${error}`)
    }
  })


});

const onImageLoad = async () => {
  await appWindow.show();
  await appWindow.requestUserAttention(UserAttentionType.Critical)
  await appWindow.setAlwaysOnTop(true);
  await appWindow.setFocus();
  await info!(">>>>>>>>>>>>>>>>>>>>>>>>>>Window show!>>>>>>>>>>>>>>>>>>>>>");
};

const handleMouseDown = (e: MouseEvent) => {
  if (e.buttons === 1) {
    isDown.value = true;
    mouseDownX.value = e.clientX;
    mouseDownY.value = e.clientY;
  } else {
    appWindow.hide();
  }
};

const handleMouseMove = (e: MouseEvent) => {
  if (isDown.value) {
    isMoved.value = true;
    mouseMoveX.value = e.clientX;
    mouseMoveY.value = e.clientY;
  }
};

const handleMouseUp = async (e: MouseEvent) => {
  // await appWindow.hide();
  isDown.value = false;
  isMoved.value = false;

  const imgWidth = (imgRef.value as HTMLCanvasElement).width;
  const dpi = imgWidth / screen.width;
  const left = Math.floor(Math.min(mouseDownX.value, e.clientX) * dpi);
  const top = Math.floor(Math.min(mouseDownY.value, e.clientY) * dpi);
  const right = Math.floor(Math.max(mouseDownX.value, e.clientX) * dpi);
  const bottom = Math.floor(Math.max(mouseDownY.value, e.clientY) * dpi);
  const width = right - left;
  const height = bottom - top;

  if (width <= 0 || height <= 0) {
    console.warn('Screenshot area is too small');
    await warn('Screenshot area is too small')
    await appWindow.close();
  } else {
    await info("emit success to rust!")
    await appWindow.emit('success', {left, top, width, height});
  }
};


</script>

<template>
  <!--  <img-->
  <!--      ref="imgRef"-->
  <!--      :style="{-->
  <!--      position: 'fixed',-->
  <!--      top: 0,-->
  <!--      left: 0,-->
  <!--      width: '100%',-->
  <!--      userSelect: 'none'-->
  <!--    }"-->
  <!--      :src="imgUrl"-->
  <!--      :draggable="false"-->
  <!--      alt="fail!"-->
  <!--      @load="onImageLoad"/>-->
  <canvas
      id="canvas"
      style="position: fixed;top: 0;left:0;width: 100%;height: 100%;  user-select: none;overflow: hidden;"
      draggable="false"
      ref="imgRef"
  >

  </canvas>

  <div
      :class="{ 'selection-box': true, 'hidden': !isMoved }"
      :style="{
      top: `${Math.min(mouseDownY, mouseMoveY)}px`,
      left: `${Math.min(mouseDownX, mouseMoveX)}px`,
      bottom: `${screen.height - Math.max(mouseDownY, mouseMoveY)}px`,
      right: `${screen.width - Math.max(mouseDownX, mouseMoveX)}px`
    }"
  />
  <div
      class="overlay"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
  ></div>
</template>

<style scoped>
canvas {
  display: block;
}

.selection-box {
  position: fixed;
  background-color: rgba(32, 128, 240, 0.125); /* #2080f020 with alpha */
  border: 1px solid #0ea5e9; /* Tailwind's sky-500 color */
}

.hidden {
  display: none;
}

.overlay {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  cursor: crosshair;
  user-select: none;
}
</style>