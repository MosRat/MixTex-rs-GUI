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
            reader.onload = function(event) {
                const arrayBuffer = event.target.result;
                const uint8Array = new Uint8Array(arrayBuffer);
                invoke('set_screenshot', uint8Array);
            };
            reader.readAsArrayBuffer(blob);
        }
    }
}




const initTauri = async () => {
    const tauri = window.__TAURI__
    const {listen} = tauri.event
    const {invoke} = tauri.core
    const {getCurrentWindow} = tauri.window
    console.log(tauri)

    // fix tauri bug https://github.com/tauri-apps/tauri/issues/8632#issuecomment-975607891
    await getCurrentWindow().show();
    await getCurrentWindow().setDecorations(true);


    // 假设显示尺寸固定为 200x200
    const displayWidth = 300;
    const displayHeight = 100;

    // 创建 canvas 元素
    const canvas = document.getElementById("screenshot");
    const context = canvas.getContext('2d');

    await listen('image_arrive', async event => {
        const {w, h} = event.payload
        await getCurrentWindow().setFocus()
        console.log("image_arrive",w, h)

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

    document.addEventListener('paste', function(event) {
        const items = event.clipboardData.items;
        console.log(event)
        processClipboardImage(items);
    });



}

document.addEventListener('DOMContentLoaded', onLoad)
