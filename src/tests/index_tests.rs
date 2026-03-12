/// 索引命令测试
use crate::commands::index::{execute, IndexConfig};
use crate::IndexAction;
use std::path::PathBuf;

#[test]
fn test_index_config_auto_init() {
    let config = IndexConfig::auto_init();

    assert!(config.initialized);

    // Linux 系统应该包含 /home
    #[cfg(target_os = "linux")]
    {
        assert!(config.indexed_paths.iter().any(|p| p == "/home"));
        assert!(config.excluded_paths.contains(&"/proc".to_string()));
        assert!(config.excluded_paths.contains(&"/sys".to_string()));
        assert!(config.excluded_paths.contains(&"/dev".to_string()));
    }
}

#[test]
fn test_index_config_save_load() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test-index-config.yaml");

    // 创建配置
    let mut config = IndexConfig::auto_init();

    // 修改配置路径
    config.indexed_paths = vec!["/test/path".to_string()];
    config.excluded_paths = vec!["/test/exclude".to_string()];

    // 保存到临时文件（这里简化测试，实际使用 get_config_path）
    let content = serde_yaml::to_string(&config).expect("Failed to serialize");
    std::fs::write(&config_path, content).expect("Failed to write");

    // 加载配置
    let content = std::fs::read_to_string(&config_path).expect("Failed to read");
    let loaded: IndexConfig = serde_yaml::from_str(&content).expect("Failed to deserialize");

    assert_eq!(loaded.indexed_paths, vec!["/test/path"]);
    assert_eq!(loaded.excluded_paths, vec!["/test/exclude"]);
}

#[test]
fn test_index_action_status() {
    // 测试 Status 命令
    let action = IndexAction::Status;
    let result = execute(&action);

    // 应该成功执行
    assert!(result.is_ok());
}

#[test]
fn test_index_action_list() {
    // 测试 List 命令
    let action = IndexAction::List;
    let result = execute(&action);

    assert!(result.is_ok());
}

#[test]
fn test_exclude_action_list() {
    // 测试 Exclude List 命令
    let action = IndexAction::Exclude {
        action: crate::ExcludeAction::List,
    };
    let result = execute(&action);

    assert!(result.is_ok());
}
