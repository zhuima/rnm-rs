use thiserror::Error;

#[derive(Error, Debug)]
pub enum RnmError {
    #[error("网络请求失败: {0}")]
    NetworkRequestError(#[from] reqwest::Error),

    #[error("文件 I/O 错误: {0}")]
    FileIoError(#[from] std::io::Error),

    #[error("解压 .tar.gz 文件失败: {0}")]
    TarGzFileError(String),

    #[error("解压 .zip 文件失败: {0}")]
    ZipFileError(String),

    #[error("无法获取用户 Home 目录: {0}")]
    HomeDirError(String),

    #[error("无效的 Node.js 版本: {0}")]
    InvalidNodeVersion(String),

    #[error("HTML 解析失败: {0}")]
    HtmlParseError(String),

    #[error("不支持的平台: os: {0}, arch: {1}")]
    UnsupportedPlatform(String, String),

    #[error("不支持的架构: {0}")]
    UnsupportedArch(String),

    #[error("版本 '{0}' 已安装")]
    AlreadyInstalled(String),

    #[error("版本 '{0}' 未安装")]
    NotInstalled(String),
}

// 定义一个统一的Result类型
pub type Result<T> = std::result::Result<T, RnmError>;
