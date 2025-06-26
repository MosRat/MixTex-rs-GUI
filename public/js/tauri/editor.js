/*
 * Copyright (c) 2024. MosRat
 * All rights reserved.
 *
 * Project: fast-writer
 * File Name: editor.js
 * Author: MosRat (work@whl.moe)
 * Description:
 */

function processClipboardImage(items) {
    const tauri = window.__TAURI__
    const {invoke} = tauri.core
    for (let i = 0; i < items.length; i++) {
        const blob = items[i].getAsFile();
        if (blob && (blob.type === "image/png" || blob.type === "image/jpeg")) {
            const reader = new FileReader();
            reader.onload = function (event) {
                const arrayBuffer = event.target.result;
                const uint8Array = new Uint8Array(arrayBuffer);
                invoke('set_screenshot', uint8Array);
            };
            reader.readAsArrayBuffer(blob);
        }
    }
}

async function processTextareaInput() {
    const tauri = window.__TAURI__
    const { invoke } = tauri.core
    // 获取textarea元素
    const textarea = document.getElementById('txta_input');

    if (!textarea) {
        console.error('Textarea with id "txta_input" not found');
        return null;
    }

    // 获取输入内容
    const inputContent = textarea.value;

    try {
        // 调用invoke函数（假设已定义）
        return await invoke("convert_doc", {latexContent:inputContent});
    } catch (error) {
        console.error('Error in invoke convert_doc:', error);
        throw error;
    }
}
if (typeof window !== 'undefined') {
    window.processTextareaInput = processTextareaInput;
}

const initTauri = async () => {
    const tauri = window.__TAURI__
    const { invoke } = tauri.core

    const { listen } = tauri.event
    const { getCurrentWindow,Effect } = tauri.window
    const { revealItemInDir } = tauri.opener;
    const { appConfigDir } = tauri.path
    console.log(tauri)

    // fix tauri bug https://github.com/tauri-apps/tauri/issues/8632#issuecomment-975607891
    await getCurrentWindow().show();
    await getCurrentWindow().setFocus();
    // await getCurrentWindow().setDecorations(true);
    // await getCurrentWindow().setEffects({effects:[Effect.TabbedLight,Effect.Mica]});


    // 假设显示尺寸固定为 200x200
    const displayWidth = 300;
    const displayHeight = 100;

    // 创建 canvas 元素
    const canvas = document.getElementById("screenshot");
    const context = canvas.getContext('2d');

    await listen('image_arrive', async event => {
        const {w, h} = event.payload
        await getCurrentWindow().setFocus()
        console.log("image_arrive", w, h)

        const f = Math.max(w / displayWidth, h / displayHeight)


        const fw = w / f
        const fh = h / f


        const imgBuffer = await invoke("get_screenshot")
        const imageData = new ImageData(new Uint8ClampedArray(imgBuffer), w, h);

        // 将原始图像缩放到固定尺寸并绘制到 canvas 上
        const tempCanvas = document.createElement('canvas');
        tempCanvas.width = w;
        tempCanvas.height = h;
        const tempContext = tempCanvas.getContext('2d');
        tempContext.putImageData(imageData, 0, 0);

        // 缩放并绘制到目标 canvas
        context.clearRect(0, 0, displayWidth, displayHeight);
        context.drawImage(tempCanvas, 0, 0, fw, fh);

        tempCanvas.remove()
    })


}

const onLoad = async () => {
    console.log(window.location.href, "load scripts")
    await initTauri()

    document.addEventListener('paste', function (event) {
        const items = event.clipboardData.items;
        console.log(event)
        processClipboardImage(items);
    });

    document.getElementById("img-canvas").onclick = () => window.__TAURI__.window.getCurrentWindow().emit('select_img')
    document.getElementById("btn_config").onclick = async () => await window.__TAURI__.opener.revealItemInDir(await window.__TAURI__.path.appConfigDir() + "\\custom.js")

}

document.addEventListener('DOMContentLoaded', onLoad)
