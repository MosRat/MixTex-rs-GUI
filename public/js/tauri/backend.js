window.backend="mixtex"
window.sl_token= localStorage.getItem('sl_token') ?? ""


const initBackend = async ()=>{
    const backendSelect = document.getElementById("backend")
    const slTokenInput = document.getElementById("sl_token")

    slTokenInput.value = localStorage.getItem('sl_token')

    backendSelect.addEventListener("change",async ()=>{
        window.backend = backendSelect.value
        if (window.backend==="sl"){
            slTokenInput.style.display = "block"
        }else {
            slTokenInput.style.display = "none"
        }
        console.log(`switch backend to ${window.backend}`)
    })
    backendSelect.addEventListener("blur",async ()=>{
        console.log("blur!")
        localStorage.setItem("sl_token",slTokenInput.value)
        window.sl_token = slTokenInput.value
        console.log(slTokenInput.value)
    })
}

document.addEventListener("DOMContentLoaded",initBackend)