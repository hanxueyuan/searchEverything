use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("文件未找到：{0}")]
    NotFound(String),
    
    #[error("权限不足：{0}")]
    PermissionDenied(String),
    
    #[error("无效路径：{0}")]
    InvalidPath(String),
    
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON 序列化错误：{0}")]
    Json(#[from] serde_json::Error),
}
