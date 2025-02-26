// 自定义替换规则 , 用于以下函数的 dictionary:
// function replaceWithDictionary(input, dictionary) {
//     let output = input;
//     for (const [key, value] of Object.entries(dictionary)) {
//         output = output.replace(key, value);
//     }
//     return output;
// }

window.replaceRules={
    // "\\(": "$$",
    // "\\)": "$$",
    // "\\[": "$$$",
    // "\\]": "$$$",
    "\\(": "", // 也可以用正则表达式
    "\\)": "",
    "\\[": "",
    "\\]": "",
}

// Simple Latex 的 token
window.sl_token_custom="you_simple_latex_token"