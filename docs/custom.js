// 自定义替换规则 , 用于以下函数的 dictionary:
// input: 生成的单个 token（字符）
// function replaceWithDictionary(input, dictionary) {
//     let output = input;
//     for (const [key, value] of Object.entries(dictionary)) {
//         output = output.replace(key, value);
//     }
//     return output;
// }
// 可以把脚本给大模型，让 AI 输出符合要求的结果
// 和 Rust 后端交互相关的代码在 public/js/tauri 可以一并给大模型
window.replaceRules={
    // "\\(": "$$",
    // "\\)": "$$",
    // "\\[": "$$$",
    // "\\]": "$$$",
    "\\(": "", // 也可以用正则表达式，符合 js 语法即可
    "\\)": "",
    "\\[": "",
    "\\]": "",
}

// Simple Latex 的 token
window.sl_token_custom="you_simple_latex_token"

window.custom_version = "v1.0.0"