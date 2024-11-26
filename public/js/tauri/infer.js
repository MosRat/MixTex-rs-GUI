const replaceChars = {
    "\\(": "$$",
    "\\)": "$$",
    "\\[": "$$$",
    "\\]": "$$$"
}

function replaceWithDictionary(input, dictionary) {
    console.log(input)
    let output = input;
    for (const [key, value] of Object.entries(dictionary)) {
        // const regex = new RegExp(key, 'g');
        output = output.replace(key, value);
    }
    return output;
}

function simulateUserInput(textarea, text) {
    if (textarea) {
        // 设置文本
        textarea.value += text;

        // 创建并触发输入事件
        const event = new Event('input', {
            bubbles: true,
            cancelable: true,
        });
        textarea.dispatchEvent(event);
    } else {
        console.error(`Element with id ${textarea} not found.`);
    }
}

const handleStart = async (event) => {
    const tauri = window.__TAURI__
    const {invoke, Channel} = tauri.core
    const textarea = document.getElementById("txta_input")

    const channel = new Channel();

    channel.onmessage = async (message) => {
        switch (message.event) {
            case "tokenArrive": {
                simulateUserInput(textarea, replaceWithDictionary(message.data.token, replaceChars))
                break
            }
            case "stop": {
                document.getElementById("stop").style.display = 'none'
                document.getElementById("start").style.display = 'block'
                break
            }
            case "err": {
                console.error(`Infer fail due to ${message?.data?.err}!`)
                simulateUserInput(textarea, message?.data?.err)
                document.getElementById("stop").style.display = 'none'
                document.getElementById("start").style.display = 'block'
                break
            }
        }
    };
    document.getElementById("stop").style.display = 'block'
    document.getElementById("start").style.display = 'none'
    console.log(window.sl_token)
    await invoke("generate", {
        ch: channel,
        backend: window.backend,
        token: window.sl_token,
    })
}

const handleStop = async (event) => {
    const tauri = window.__TAURI__
    const {emit} = tauri.event

    await emit("stop")
    document.getElementById("stop").style.display = 'none'
    document.getElementById("start").style.display = 'block'
}

const handleStrip = async (_event) => {
    let textArea = document.getElementById("txta_input");
    textArea.value = textArea.value.replace(/\$/g, "").trim()

    const event = new Event('input', {
        bubbles: true,
        cancelable: true,
    });
    textArea.dispatchEvent(event);
}

const initInfer = async () => {
    document.getElementById("start").onclick = handleStart
    document.getElementById("stop").onclick = handleStop
    document.getElementById("strip").onclick = handleStrip

    document.getElementById("stop").style.display = 'none'
    document.getElementById("start").style.display = 'block'

}

document.addEventListener("DOMContentLoaded", initInfer)