<script setup lang="ts">
import {nextTick, ref, watch} from 'vue'
import {invoke} from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog';
import {appWindow} from "@tauri-apps/api/window";
import renderMathInElement from 'katex/dist/contrib/auto-render.mjs';


const decode_text = ref('')
const text = ref('')
const running = ref(false)
// const dropZoneBorderColor = ref("")
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const katexRef = ref<HTMLDivElement | null>(null);

// 处理文件拖放
appWindow.onFileDropEvent(event => {
  if (event.payload.type === "drop") {
    text.value = event.payload.paths[0]
  }
})



// 处理更新和渲染 latex
const scrollToBottom = () => {
  if (textareaRef.value) {
    textareaRef.value.scrollTop = textareaRef.value.scrollHeight;
  }
  if (katexRef.value) {
    katexRef.value.scrollTop = katexRef.value.scrollHeight;
  }
};

watch(decode_text, () => {
  nextTick(() => {
    renderMathInElement(katexRef.value, {});
    scrollToBottom()

  });
}, {immediate: true});

// 处理点击和推理
type Payload = {
  token: string
}

const handleFileSelect = async () => {
    const selected = await open({
    multiple: false,
    filters: [{
      name: 'Image',
      extensions: ['png', 'jpeg','jpg']
    }]
  });
  if (Array.isArray(selected)) {
    // user selected multiple files
  } else if (selected === null) {
    // user cancelled the selection
  } else {
    text.value = selected;
    console.log(selected);
  }
}

const handleInfer = async () => {
  running.value = true;
  let handle = await appWindow.listen("result", payload => {
    decode_text.value += (payload.payload as Payload).token;
  })
  await invoke('inference', {path: text.value})
  handle()
  running.value = false
}
const handleStop = async () => {
  await appWindow.emit("stop")
  await appWindow.once("infer_stop", (_) => {
    running.value = false
  })
}

const handleClear = async () => {
  decode_text.value = ""
}



// const handleDragOver = (event: any) => {
//   // 阻止默认的拖放行为
//   event.preventDefault()
//   dropZoneBorderColor.value = '#2da74e'
// }
//
// const handleDragLeave = (event: any) => {
//   // 阻止默认的拖放行为
//   event.preventDefault()
//   dropZoneBorderColor.value = '#d9dbde'
// }
//
// const handleDrop = async (event: any) => {
//   event.preventDefault()
//   dropZoneBorderColor.value = '#d9dbde'
//
//   const item = event.dataTransfer.items[0]
//   // DataTransferItem 对象提供了webkitGetAsEntry()方法，用于获取文件数据项的文件系统入口对象
//   // 该方法会返回两种格式对象：FileEntry（文件）、DirectoryEntry（文件夹）
//   const entry = item.webkitGetAsEntry()
//   // 使用isFile字段判断是否是文件,因为是第一层文件则没有文件夹，需要设置属性isFirstFile为true方便后面拼接路径使用
//   if (entry.isFile && entry.name) {
//     text.value = entry.fullPath
//   }
//
// }


</script>

<template>
  <div class="container">
    <div class="v-container">
      <div class="output">
        <textarea ref="textareaRef" class="text-content" :value="decode_text" readonly style="resize: none;"></textarea>
        <div ref="katexRef" class="latex">
          {{ decode_text }}
        </div>
      </div>
      <div class="h-container">
        <input
          class="input-content"
          type="text"
          v-model="text"
      >
        <div class="r-button" @click="handleFileSelect">Select</div>
      </div>
<!--      <div class="drag-file"-->
<!--          :style="`border-color:${dropZoneBorderColor}`"-->
<!--           @dragover="handleDragOver"-->
<!--           @dragleave="handleDragLeave"-->
<!--           @drop="handleDrop">-->
<!--        <p>拖动文件至此</p>-->
<!--      </div>-->
    </div>
    <div>
      <button @click="handleInfer" v-if="!running" class="button" id="run-button">Decode</button>
      <button @click="handleStop" v-if="running" class="button" id="stop-button">Stop</button>
      <button @click="handleClear" :disabled="running" class="button" id="clear-button">Clear</button>
    </div>
  </div>
</template>

<style scoped>

@font-face {
  font-family: 'JetBrains Mono'; /* 自定义的字体名称 */
  src: url('/fonts/JetBrainsMono-SemiBold.woff2') format('woff2'); /* 指定 .woff2 文件的路径和格式 */
  font-weight: normal; /* 字体的权重 (例如 normal, bold) */
  font-style: normal; /* 字体的样式 (例如 normal, italic) */
}

/* 移除 input 和 textarea 聚焦时的白框 */
input:focus,
textarea:focus {
  outline: none; /* 移除默认的聚焦样式 */
  box-shadow: none; /* 移除可能存在的阴影效果 */
}


input, button, textarea, .latex,.drag-file,.r-button{
  border-radius: 8px;
  border: 2px solid rgba(37, 37, 37, 0.47);
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: rgba(255, 255, 255, 0.21);
  transition: border-color 0.25s;
  box-shadow: 0 1px 1px rgba(0, 0, 0, 0.2);
}
.r-button{
  border-radius: 2px;
  background-color: rgba(37, 37, 37, 0.47);
  font-weight: bolder;
  width: 10vw;
  padding-block: 0.25em;
  box-sizing: border-box;
}

textarea {
  padding: 6px;
  font-family: "JetBrains Mono", monospace;
  font-size: 0.8em;
}

button {
  border-radius: 14px;
  background-color: rgba(37, 37, 37, 0.47);
  font-weight: bolder;
}

button:hover {
  background-color: rgba(132, 131, 131, 0.47);
}

.container {
  display: flex;
  justify-content: center;
  align-items: center;
  max-height: 95vh;
}

.v-container {
  display: flex;
  flex-direction: column;
}

.h-container{
  display: flex;
  flex-direction: row;
  max-width: 85vw;
}




.output {
  display: flex;
  flex-direction: row;
  max-width: 85vw;
}

.text-content {
  height: 60vh;
  width: 42.5vw;
}

.latex {
  height: 60vh;
  width: 42.5vw;
  overflow: scroll;
  padding: 6px;
  text-align: left;
}

.input-content {
  min-height: 2em;
  width: 75vw;
}

.drag-file{
  min-height: 10vh;
  border: 2px solid rgba(37, 37, 37, 0.47);
}

.button {
  margin-top: 1em;
  margin-inline: 1em;
  min-width: 20vw;
  min-height: 2em;
}

#run-button:hover {
  filter: drop-shadow(0 0 1em #747bff);
}

#stop-button:hover {
  filter: drop-shadow(0 0 1em #fd3d4e);
}

#clear-button:hover {
  filter: drop-shadow(0 0 1em #02a918);
}

</style>