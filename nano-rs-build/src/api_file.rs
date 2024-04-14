use std::fs;
use std::path::{Path, PathBuf};

///迭代遍历该目录下全部rs文件
pub fn get_rs_files(files: &mut Vec<PathBuf>, path: &Path) -> std::io::Result<()> {
    //判断当前是否是文件夹
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            get_rs_files(files, &path)?;
        }
    } else if let Some(ext) = path.extension() {
        if ext == "rs" {
            files.push(path.to_path_buf());
        }
    }
    Ok(())
}
