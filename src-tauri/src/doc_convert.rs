use std::fs::File;
use std::io::Write;
use std::process::Command;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub async fn convert_doc(
    app: AppHandle,
    latex_content: String,
) -> Result<(), String> {
    // 1. 创建临时目录和文件
    let temp_dir = app.path().temp_dir().map_err(|e| format!("获取临时目录失败: {}", e))?;
    let input_path = temp_dir.join("input.tex");
    let output_path = temp_dir.join("output.docx");
    
    let latex_content = format!(r#"
    \documentclass{{rticle}}
    \usepackage{{amsmath}}
    \usepackage{{amssymb}}
    \usepackage{{amsfonts}}
    \usepackage{{cases}}
    \usepackage[UTF8]{{ctex}}
    \begin{{document}}
    
    {latex_content}
    
    \end{{document}}
    "#);
    
    // 2. 写入临时 LaTeX 文件
    File::create(&input_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?
        .write_all(latex_content.as_bytes())
        .map_err(|e| format!("写入 LaTeX 内容失败: {}", e))?;

    // 3. 调用 Pandoc sidecar 进行转换
    let sidecar_command = app
        .shell()
        .sidecar("pandoc")
        .map_err(|e| format!("初始化 Pandoc sidecar 失败: {}", e))?
        .args([
            input_path.to_str().ok_or("临时文件路径包含非法字符")?,
            "-o",
            output_path.to_str().ok_or("输出文件路径包含非法字符")?,
            "--mathml",
        ]);

    let conversion_result = sidecar_command
        .output()
        .await
        .map_err(|e| format!("Pandoc 执行失败: {}", e))?;

    if !conversion_result.status.success() {
        return Err(format!(
            "Pandoc 转换失败: {}",
            String::from_utf8_lossy(&conversion_result.stderr)
        ));
    }

    // 4. 调用 PowerShell 复制到剪贴板
    let ps_script = format!(
        r#"
        $word = New-Object -ComObject Word.Application
        $doc = $word.Documents.Open("{}")
        $doc.Content.Copy()
        $word.Quit()
        "#,
        output_path.to_str().unwrap().replace("\\", "\\\\")
    );

    let ps_status = Command::new("powershell.exe")
        .env("POWERSHELL_UPDATECHECK", "Off")
        .arg("-NonInteractive")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-NoProfile")
        .arg("-NoLogo")
        .arg("-Command")
        .arg(&ps_script)
        .status()
        .map_err(|e| format!("PowerShell 调用失败: {}", e))?;

    if !ps_status.success() {
        return Err("复制到剪贴板失败（请检查 Word 是否安装）".into());
    }

    // 5. 清理临时文件（可选）
    // 这里可以添加清理代码，或者依赖系统自动清理临时目录

    Ok(())
}