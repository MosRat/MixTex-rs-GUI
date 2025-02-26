// 自定义替换规则 , 用于以下函数的dictionary:
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
    "\\(": "",
    "\\)": "",
    "\\[": "",
    "\\]": "",
}

// Simple Latex 的 token
window.sl_token_custom="you_simple_latex_token"