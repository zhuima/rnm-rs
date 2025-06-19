use crate::error::{RnmError, Result};
use std::path::PathBuf;
use std::fs;


pub const NODE_MIRROR: &str = "https://nodejs.org/dist";
// pub const CURRENT_SYMLINK: &str = "current";


// 获取rnm的主目录， 默认为 ~/.rnm
pub fn get_rnm_dir() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or(RnmError::HomeDirError("无法获取用户 Home 目录".to_string()))?;
    let rnm_dir = PathBuf::from(&home_dir).join(".rnm");
    Ok(rnm_dir)
}


// 获取rnm的版本目录， 默认为 ~/.rnm/versions
pub fn get_versions_dir() -> Result<PathBuf> {
    let rnm_dir = get_rnm_dir()?;
    let versions_dir = rnm_dir.join("versions");
    
    // 确保目录存在
    if !versions_dir.exists() {
        fs::create_dir_all(&versions_dir)?;
    }
    
    Ok(versions_dir)
}









