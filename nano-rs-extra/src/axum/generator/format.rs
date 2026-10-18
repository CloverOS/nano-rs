//! 生成代码共用的格式化入口：先在内存中格式化，再按最终内容决定是否写入文件。

use super::write_if_changed;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// 使用项目配置格式化生成代码，返回是否实际写入了文件。
///
/// rustfmt 不可用或执行失败时告警并保留传入内容；文件系统错误仍返回给调用方。
pub(super) fn write_rust_source(path: &Path, contents: &str) -> io::Result<bool> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut formatter = Command::new("rustfmt");
    // stdin 输入没有源文件路径，通过子进程工作目录查找 rustfmt 配置，不改变调用方目录。
    formatter
        .current_dir(parent)
        .args(["--edition", "2024", "--emit", "stdout"]);
    match format_source(formatter, contents) {
        Ok(formatted) => write_if_changed(path, &formatted),
        Err(err) => {
            // 将多行诊断压成一行，保持 Cargo 构建警告的输出格式。
            println!(
                "cargo:warning=nano-rs: rustfmt failed for {}; keeping generated content: {}",
                path.display(),
                err.to_string().replace(['\r', '\n'], " ")
            );
            write_if_changed(path, contents)
        }
    }
}

/// 缓存命中时重新格式化现有文件，让旧输出和格式配置变更也能得到处理。
/// 内容不变时不重写文件，保留原修改时间。
pub(super) fn format_existing_rust_file(path: &Path) -> io::Result<bool> {
    write_rust_source(path, &fs::read_to_string(path)?)
}

/// 通过标准输入和输出格式化源码，避免 rustfmt 直接改写生成文件。
fn format_source(mut formatter: Command, contents: &str) -> io::Result<String> {
    let mut child = formatter
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().expect("rustfmt stdin is piped");
    // 并行写入 stdin 和读取 stdout/stderr，防止大文件填满管道后互相等待。
    // 写入线程结束时关闭 stdin，让 rustfmt 能读取到 EOF 并完成格式化。
    let (output, written) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || stdin.write_all(contents.as_bytes()));
        let output = child.wait_with_output();
        let written = writer
            .join()
            .map_err(|_| io::Error::other("rustfmt stdin writer panicked"))
            .and_then(|result| result);
        (output, written)
    });
    let output = output?;
    // 优先报告 rustfmt 的失败原因；提前退出时，写入线程也可能遇到 BrokenPipe。
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    written?;
    String::from_utf8(output.stdout).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}
