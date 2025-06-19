use crate::error::{RnmError, Result};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use tar::Archive;



// 解压 .tar.gz 文件
pub fn extract_tar_gz(src: &Path, dst: &Path) -> Result<()> {
    // 打开源文件
    let mut file = File::open(src).map_err(|e| RnmError::FileIoError(e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| RnmError::FileIoError(e))?;

    let decoder = GzDecoder::new(&buffer[..]);
    let mut archive = Archive::new(decoder);
    archive.unpack(dst).map_err(|e| RnmError::TarGzFileError(e.to_string()))?;
    Ok(())
}


// 解压 .zip 文件
pub fn extract_zip(src: &Path, dst: &Path) -> Result<()> {
    let mut file = File::open(src).map_err(|e| RnmError::FileIoError(e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| RnmError::FileIoError(e))?;

    let reader = std::io::Cursor::new(buffer);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| RnmError::ZipFileError(e.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| RnmError::ZipFileError(e.to_string()))?;
        let outpath = dst.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| RnmError::FileIoError(e))?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p).map_err(|e| RnmError::FileIoError(e))?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| RnmError::FileIoError(e))?;
            io::copy(&mut file, &mut outfile).map_err(|e| RnmError::FileIoError(e))?;
        }
    }
    Ok(())
}


