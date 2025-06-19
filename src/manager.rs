use crate::config::{get_versions_dir, NODE_MIRROR};
use crate::error::{Result, RnmError};
use crate::utils::{extract_tar_gz as decompress_tar_gz, extract_zip as decompress_zip};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use scraper::{Html, Selector};
use semver::Version;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;


#[cfg(unix)]
use std::os::unix::fs as unix_fs;

#[cfg(windows)]
use std::os::windows::fs as windows_fs;

pub struct Manager {
    client: Client,
    versions_dir: PathBuf,
    #[allow(dead_code)]
    original_user: Option<String>,
}

impl Manager {
    pub fn new() -> Result<Self> {
        let versions_dir = get_versions_dir()?;
        
        // 获取 SUDO_USER 环境变量，这是原始用户的用户名
        let original_user = env::var("SUDO_USER").ok();
        
        // 如果是以 sudo 运行，则使用原始用户的目录
        let versions_dir = if let Some(user) = &original_user {
            if cfg!(unix) {
                PathBuf::from(format!("/home/{}", user)).join(".rnm/versions")
            } else {
                versions_dir
            }
        } else {
            versions_dir
        };

        // 确保目录存在
        if !versions_dir.exists() {
            // 如果是 sudo 用户，需要创建目录并修改所有权
            if let Some(user) = &original_user {
                fs::create_dir_all(&versions_dir)?;
                // 使用 chown 命令修改目录所有权
                let output = std::process::Command::new("chown")
                    .arg("-R")
                    .arg(format!("{}:{}", user, user))
                    .arg(&versions_dir)
                    .output()?;
                
                if !output.status.success() {
                    println!("警告: 修改目录所有权失败: {:?}", String::from_utf8_lossy(&output.stderr));
                }
            } else {
                fs::create_dir_all(&versions_dir)?;
            }
        }

        Ok(Self {
            client: Client::new(),
            versions_dir,
            original_user,
        })
    }

    pub async fn list_remote(&self, version_filter: Option<&str>) -> Result<Vec<String>> {
        println!("正在从 {} 获取版本信息...", NODE_MIRROR);
        let response = self.client.get(NODE_MIRROR).send().await?.text().await?;

        let document = Html::parse_document(&response);
        let selector = Selector::parse("a").map_err(|e| RnmError::HtmlParseError(e.to_string()))?;

        let mut versions: Vec<Version> = document
            .select(&selector)
            .filter_map(|el| {
                let text = el.text().collect::<String>();
                text.trim_end_matches('/')
                    .strip_prefix("v")
                    .and_then(|v| Version::parse(v).ok())
            })
            .collect();

        versions.sort();
        versions.reverse();

        // 如果指定了版本过滤
        if let Some(filter) = version_filter {
            let parts: Vec<&str> = filter.split('.').collect();
            let filtered_versions: Vec<&Version> = versions.iter()
                .filter(|v| {
                    match parts.len() {
                        1 => v.major.to_string() == parts[0],
                        2 => v.major.to_string() == parts[0] && v.minor.to_string() == parts[1],
                        3 => v.major.to_string() == parts[0] && v.minor.to_string() == parts[1] && v.patch.to_string() == parts[2],
                        _ => false,
                    }
                })
                .collect();

            if filtered_versions.is_empty() {
                return Ok(vec![format!("未找到匹配版本 '{}'", filter)]);
            }

            return Ok(filtered_versions.iter()
                .map(|v| format!("\x1b[1;32m{}\x1b[0m", v))
                .collect());
        }

        // 按主版本号分组显示
        let mut grouped_versions: std::collections::BTreeMap<u64, Vec<&Version>> = std::collections::BTreeMap::new();
        for version in &versions {
            grouped_versions.entry(version.major).or_default().push(version);
        }

        // 只保留每个主版本号下最新的几个版本
        let mut result = Vec::new();
        for (major, versions) in grouped_versions.iter().rev().take(3) {  // 只显示最新的3个主版本
            result.push(format!("\n\x1b[1;32mNode.js v{}.x\x1b[0m:", major));
            for version in versions.iter().take(5) {  // 每个主版本只显示最新的5个版本
                result.push(format!("  {}", version));
            }
            if versions.len() > 5 {
                result.push(format!("  \x1b[90m... and {} more\x1b[0m", versions.len() - 5));
            }
        }

        Ok(result)
    }

    /// 获取所有已安装的版本列表
    fn get_installed_versions(&self) -> Result<Vec<String>> {
        println!("正在检查版本目录: {:?}", self.versions_dir);
        
        if !self.versions_dir.exists() {
            println!("版本目录不存在，正在创建: {:?}", self.versions_dir);
            fs::create_dir_all(&self.versions_dir)?;
            return Ok(vec![]);
        }

        if !self.versions_dir.is_dir() {
            return Err(RnmError::FileIoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{:?} 不是一个目录", self.versions_dir),
            )));
        }

        let mut versions = Vec::new();
        let entries = match fs::read_dir(&self.versions_dir) {
            Ok(entries) => entries,
            Err(e) => {
                println!("读取目录失败: {:?}, error: {}", self.versions_dir, e);
                return Err(RnmError::FileIoError(e));
            }
        };

        for entry in entries {
            match entry {
                Ok(entry) => {
                    match entry.file_type() {
                        Ok(file_type) => {
                            if file_type.is_dir() {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.starts_with("node-v") {
                                        let version = name.trim_start_matches("node-v").to_string();
                                        versions.push(version);
                                    }
                                }
                            }
                        }
                        Err(e) => println!("获取文件类型失败: {:?}, error: {}", entry.path(), e),
                    }
                }
                Err(e) => println!("读取目录项失败: error: {}", e),
            }
        }
        versions.sort();
        println!("找到已安装版本: {:?}", versions);
        Ok(versions)
    }

    pub async fn list_local(&self) -> Result<Vec<String>> {
        println!("Checking versions directory: {:?}", self.versions_dir);
        
        if !self.versions_dir.exists() {
            println!("Creating versions directory as it doesn't exist");
            fs::create_dir_all(&self.versions_dir)?;
            return Ok(vec![]);
        }

        // 确保目录存在且可访问
        if !self.versions_dir.is_dir() {
            return Err(RnmError::FileIoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{:?} is not a directory", self.versions_dir),
            )));
        }

        let versions = self.get_installed_versions()?;
        
        if versions.is_empty() {
            println!("No versions found in the directory");
            return Ok(vec![]);
        }


        
        println!("Found {} installed versions: {:?}", versions.len(), versions);
        Ok(versions)
    }

    pub async fn install(&self, version: &str) -> Result<()> {
        let version_str = version.trim_start_matches('v').to_string();
        let version_dir = format!("node-v{}", version_str);
        let version_path = self.versions_dir.join(&version_dir);
        if version_path.exists() {
            return Err(RnmError::AlreadyInstalled(version_str.to_string()));
        }

        // 确定平台和架构
        let os = env::consts::OS;
        let arch = match env::consts::ARCH {
            "x86_64" => "x64",
            "aarch64" => "arm64",
            _ => return Err(RnmError::UnsupportedArch(env::consts::ARCH.to_string())),
        };

        let extension = if os == "windows" { "zip" } else { "tar.gz" };
        
        let os_name = match os {
            "linux" => "linux",
            "windows" => "win",
            "macos" => "darwin",
            _ => return Err(RnmError::UnsupportedPlatform(os.to_string(), arch.to_string())),
        };

        let filename = format!("node-v{}-{}-{}.{}", version_str, os_name, arch, extension);
        let download_url = format!("{}/v{}/{}", NODE_MIRROR, version_str, filename);

        println!("Downloading from {} ...", download_url);

        // 下载并显示进度条
        let mut response = self
            .client
            .get(&download_url)
            .send()
            .await?
            .error_for_status()?;
        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

        let temp_dir = tempfile::tempdir()?;
        let temp_file_path = temp_dir.path().join(&filename);

        let mut temp_file = File::create(&temp_file_path)?;

        while let Some(chunk) = response.chunk().await? {
            temp_file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Download complete");

        // 解压文件
        println!("Unzipping file ...");
        let extracted_dir_name = format!("node-v{}-{}-{}", version_str, os_name, arch);
        let temp_extract_path = temp_dir.path();

        if extension == "zip" {
            decompress_zip(&temp_file_path, temp_extract_path)?;
        } else {
            decompress_tar_gz(&temp_file_path, temp_extract_path)?;
        }

        // 移动文件到目标目录
        let extracted_path = temp_dir.path().join(&extracted_dir_name);
        
        if !extracted_path.exists() {
            return Err(RnmError::FileIoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("解压后的目录不存在: {:?}", extracted_path),
            )));
        }

        fs::rename(&extracted_path, &version_path)?;
        fs::remove_file(&temp_file_path)?;

        println!("Installed {} in {}", version_str, version_path.display());
        Ok(())
    }

    pub fn use_version(&self, version: &str) -> Result<()> {
        let version_str = version.trim_start_matches('v');
        let version_dir = format!("node-v{}", version_str);
        let version_path = self.versions_dir.join(&version_dir);

        // 获取已安装的版本列表
        let installed_versions = self.get_installed_versions()?;

        println!("Installed versions: {:?}", installed_versions);

        // 检查版本是否已安装
        if !version_path.exists() || !installed_versions.contains(&version_str.to_string()) {
            println!("版本 '{}' 未安装。", version_str);
            if !installed_versions.is_empty() {
                println!("当前已安装的版本：");
                for ver in &installed_versions {
                    println!("  {}", ver);
                }
            } else {
                println!("当前没有安装任何版本。");
            }
            println!("\n提示: 请使用 'rnm-rs install {}' 安装此版本，或从以上版本中选择。", version_str);
            return Err(RnmError::NotInstalled(version_str.to_string()));
        }

        let bin_path = version_path.join("bin");
        if !bin_path.exists() {
            return Err(RnmError::InvalidNodeVersion(version_str.to_string()));
        }

        let symlink_base = PathBuf::from("/usr/local/bin");
        println!(
            "切换到版本 {}。此操作需要 sudo 权限。",
            version_str
        );

        // 检查是否有足够权限
        if !has_write_permission(&symlink_base) {
            println!("权限不足。请使用 sudo 运行此命令：");
            println!("sudo rnm-rs use {}", version_str);
            return Err(RnmError::FileIoError(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "需要 sudo 权限",
            )));
        }

        for tool in ["node", "npm", "npx"] {
            let symlink_path = symlink_base.join(tool);
            let target_path = bin_path.join(tool);

            // 如果符号链接已存在，先删除
            if symlink_path.exists() {
                fs::remove_file(&symlink_path)?;
            }

            // 创建符号链接
            #[cfg(unix)]
            unix_fs::symlink(&target_path, &symlink_path)?;
            #[cfg(windows)]
            windows_fs::symlink_file(&target_path, &symlink_path)?;

            println!("{} -> {}", symlink_path.display(), target_path.display());
        }

        println!("成功切换到版本 {}", version_str);
        println!("运行 `node -v` 验证安装。");
        Ok(())
    }

    pub async fn current(&self) -> Result<Option<String>> {
        let node_path = PathBuf::from("/usr/local/bin/node");
        
        // 如果软链接不存在，直接返回 None
        if !node_path.exists() {
            return Ok(None);
        }

        // 尝试读取软链接，如果失败直接返回 None
        let real_path = match fs::read_link(&node_path) {
            Ok(path) => path,
            Err(_) => return Ok(None),
        };

        let version_dir_str = match self.versions_dir.to_str() {
            Some(s) => s,
            None => return Ok(None),
        };

        let path_str = match real_path.to_str() {
            Some(s) => s,
            None => return Ok(None),
        };

        // 如果路径不包含我们的版本目录，说明不是由我们管理的
        if !path_str.contains(version_dir_str) {
            return Ok(None);
        }

        // 从路径中提取版本号
        let version = path_str
            .split(version_dir_str)
            .nth(1)
            .and_then(|s| s.split('/').nth(1))
            .and_then(|s| s.strip_prefix("node-v"))
            .map(|s| s.to_string());

        Ok(version)
    }

    pub async fn uninstall(&self, version: &str) -> Result<()> {
        let version_str = version.trim_start_matches('v');
        let version_dir = format!("node-v{}", version_str);
        let version_path = self.versions_dir.join(&version_dir);
        if !version_path.exists() {
            return Err(RnmError::NotInstalled(version.to_string()));
        }
        fs::remove_dir_all(&version_path)?;
        println!(
            "Successfully uninstalled version {}",
            version_path.display()
        );
        Ok(())
    }
}

fn has_write_permission(path: &PathBuf) -> bool {
    match fs::metadata(path) {
        Ok(_) => {
            // 尝试创建临时文件来测试写入权限
            let temp_file = path.join(".rnm_test_permissions");
            let result = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&temp_file);
            
            // 如果临时文件创建成功，删除它
            if result.is_ok() {
                let _ = fs::remove_file(&temp_file);
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
