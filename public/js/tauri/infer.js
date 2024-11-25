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
        console.log(`got event ${JSON.stringify(message)} ${message?.data?.token}`);
        switch (message.event) {
            case "tokenArrive":{
                simulateUserInput(textarea,message.data.token)
                break
            }
            case "stop":{
                document.getElementById("stop").style.display = 'none'
                document.getElementById("start").style.display = 'block'
                break
            }
            case "err":{
                console.error("Infer fail due to no image!")
                simulateUserInput(textarea,"no image!")
                document.getElementById("stop").style.display = 'none'
                document.getElementById("start").style.display = 'block'
                break
            }
        }
    };
    document.getElementById("stop").style.display = 'block'
    document.getElementById("start").style.display = 'none'
    await invoke("generate", {
        ch: channel
    })
}

const handleStop = async (event) => {
    const tauri = window.__TAURI__
    const {emit} = tauri.event

    await emit("stop")
    document.getElementById("stop").style.display = 'none'
    document.getElementById("start").style.display = 'block'
}

const init = async () => {
    document.getElementById("start").onclick = handleStart
    document.getElementById("stop").onclick = handleStop

    document.getElementById("stop").style.display = 'none'
    document.getElementById("start").style.display = 'block'

}

document.addEventListener("DOMContentLoaded", init)