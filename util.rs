use std::path::Path;
use std::fs;

pub fn list_files(path: impl AsRef<Path>, ext: &impl AsRef<str>) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let str_ext = ext.as_ref();
    if path.as_ref().is_dir() {
        let paths = fs::read_dir(&path);
        if let Ok(paths) = paths {
            for path_result in paths {
                if let Ok(path_result) = path_result {
                    if let Ok(file_type) = path_result.file_type() {
                        if file_type.is_dir() {
                             res.append(&mut list_files(path_result.path(), ext))
                        } else if file_type.is_file() {
                            if let Some(curr_ext) = path_result.path().as_path().extension() {
                                let curr_ext = curr_ext.to_str().unwrap().to_string();
                                if str_ext.contains(&curr_ext) {
                                    res.push(path.as_ref().to_str().unwrap().to_string())
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        if let Some(curr_ext) = path.as_ref().extension() {
            let curr_ext = curr_ext.to_str().unwrap().to_string();
            if str_ext.contains(&curr_ext) {
                res.push(path.as_ref().to_str().unwrap().to_string())
            }
        }
    }
    res
}