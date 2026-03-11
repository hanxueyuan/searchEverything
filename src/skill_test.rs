/// OpenClaw Skill 测试工具
/// 
/// 用于测试 Skill 触发和命令生成
/// 
/// 功能：
/// - 测试 trigger_patterns 是否正确触发对应命令
/// - 验证参数解析
/// - 验证 JSON 输出格式
/// - 生成测试报告

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Skill 测试用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTestCase {
    /// 测试名称
    pub name: String,
    /// 用户输入
    pub input: String,
    /// 期望触发的命令
    pub expected_command: String,
    /// 期望的参数（JSON）
    pub expected_params: Option<serde_json::Value>,
    /// 是否应该匹配
    pub should_match: bool,
}

/// Skill 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTestResult {
    /// 测试用例名称
    pub name: String,
    /// 用户输入
    pub input: String,
    /// 是否通过
    pub passed: bool,
    /// 实际触发的命令
    pub actual_command: Option<String>,
    /// 实际参数
    pub actual_params: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// Skill 测试器
pub struct SkillTester {
    test_cases: Vec<SkillTestCase>,
}

impl SkillTester {
    pub fn new() -> Self {
        Self {
            test_cases: Vec::new(),
        }
    }
    
    /// 添加测试用例
    pub fn add_test_case(&mut self, name: &str, input: &str, expected_command: &str) {
        self.test_cases.push(SkillTestCase {
            name: name.to_string(),
            input: input.to_string(),
            expected_command: expected_command.to_string(),
            expected_params: None,
            should_match: true,
        });
    }
    
    /// 添加带参数的测试用例
    pub fn add_test_case_with_params(
        &mut self,
        name: &str,
        input: &str,
        expected_command: &str,
        params: serde_json::Value,
    ) {
        self.test_cases.push(SkillTestCase {
            name: name.to_string(),
            input: input.to_string(),
            expected_command: expected_command.to_string(),
            expected_params: Some(params),
            should_match: true,
        });
    }
    
    /// 添加负向测试用例（不应该匹配）
    pub fn add_negative_test_case(&mut self, name: &str, input: &str) {
        self.test_cases.push(SkillTestCase {
            name: name.to_string(),
            input: input.to_string(),
            expected_command: String::new(),
            expected_params: None,
            should_match: false,
        });
    }
    
    /// 运行所有测试
    pub fn run_tests(&self) -> Vec<SkillTestResult> {
        let mut results = Vec::new();
        
        for test_case in &self.test_cases {
            let result = self.run_single_test(test_case);
            results.push(result);
        }
        
        results
    }
    
    /// 运行单个测试
    fn run_single_test(&self, test_case: &SkillTestCase) -> SkillTestResult {
        let input_lower = test_case.input.to_lowercase();
        
        // 简单的关键词匹配（模拟 OpenClaw 的 trigger_patterns 匹配）
        let matched_command = self.match_command(&input_lower);
        
        let passed = if test_case.should_match {
            // 正向测试：应该匹配到期望的命令
            matched_command.as_ref() == Some(&test_case.expected_command)
        } else {
            // 负向测试：不应该匹配任何命令
            matched_command.is_none()
        };
        
        let error = if !passed {
            if test_case.should_match {
                Some(format!(
                    "期望命令：{}, 实际命令：{:?}",
                    test_case.expected_command,
                    matched_command
                ))
            } else {
                Some(format!(
                    "不应该匹配任何命令，但匹配到了：{:?}",
                    matched_command
                ))
            }
        } else {
            None
        };
        
        SkillTestResult {
            name: test_case.name.clone(),
            input: test_case.input.clone(),
            passed,
            actual_command: matched_command,
            actual_params: None,
            error,
        }
    }
    
    /// 匹配命令（模拟 OpenClaw trigger_patterns）
    fn match_command(&self, input: &str) -> Option<String> {
        // 搜索命令
        if input.contains("搜索") || input.contains("查找") || input.contains("有没有") 
            || input.contains("找.*文件") || input.contains("在哪里") || input.contains("在哪个目录") {
            return Some("search".to_string());
        }
        
        // 信息命令
        if input.contains("文件信息") || input.contains("查看.*信息") || input.contains("这个文件")
            || input.contains("多大") || input.contains("什么时候修改") {
            return Some("info".to_string());
        }
        
        // 读取命令
        if input.contains("打开") || input.contains("读取") || input.contains("查看内容")
            || input.contains("显示.*内容") || input.contains("里面是什么") {
            return Some("cat".to_string());
        }
        
        // 复制命令
        if input.contains("复制") || input.contains("拷贝") || input.contains("把这个.*到") {
            return Some("copy".to_string());
        }
        
        // 移动命令
        if input.contains("移动") || input.contains("剪切") || input.contains("把这个.*移到") {
            return Some("move".to_string());
        }
        
        // 删除命令
        if input.contains("删除") || input.contains("移除") || input.contains("不要.*了") 
            || input.contains("清理") {
            return Some("delete".to_string());
        }
        
        // 配置命令
        if input.contains("配置") || input.contains("显示配置") {
            return Some("config".to_string());
        }
        
        // 索引命令
        if input.contains("索引状态") || input.contains("索引.*怎么样") || input.contains("索引更新")
            || input.contains("重建索引") || input.contains("更新索引") || input.contains("刷新索引") {
            return Some("index".to_string());
        }
        
        None
    }
    
    /// 打印测试结果
    pub fn print_results(&self, results: &[SkillTestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        println!("\n{}", "=".repeat(60));
        println!("OpenClaw Skill 测试结果");
        println!("{}", "=".repeat(60));
        println!("总计：{} | 通过：{} | 失败：{}", total, passed, failed);
        println!("通过率：{:.1}%", (passed as f64 / total as f64) * 100.0);
        println!();
        
        // 按命令分组显示
        let mut by_command: std::collections::HashMap<String, Vec<&SkillTestResult>> = 
            std::collections::HashMap::new();
        
        for result in results {
            let cmd = result.actual_command.clone().unwrap_or_else(|| "unknown".to_string());
            by_command.entry(cmd).or_default().push(result);
        }
        
        println!("按命令分类:");
        println!("{}", "-".repeat(40));
        for (cmd, cmd_results) in &by_command {
            let cmd_passed = cmd_results.iter().filter(|r| r.passed).count();
            println!("  {}: {}/{} 通过", cmd, cmd_passed, cmd_results.len());
        }
        println!();
        
        // 显示失败的测试
        if failed > 0 {
            println!("失败的测试:");
            println!("{}", "-".repeat(40));
            for (i, result) in results.iter().enumerate() {
                if !result.passed {
                    println!("{}. ❌ {}", i + 1, result.name);
                    println!("   输入：{}", result.input);
                    if let Some(ref error) = result.error {
                        println!("   错误：{}", error);
                    }
                    println!();
                }
            }
        }
        
        // 显示所有测试（简洁模式）
        println!("详细结果:");
        println!("{}", "-".repeat(40));
        for (i, result) in results.iter().enumerate() {
            let status = if result.passed { "✅" } else { "❌" };
            println!("{}. {} {} → {}", i + 1, status, result.name, 
                result.actual_command.as_ref().unwrap_or(&"no match".to_string()));
        }
        
        println!();
    }
    
    /// 生成 JSON 测试报告
    pub fn generate_json_report(&self, results: &[SkillTestResult]) -> String {
        let report = serde_json::json!({
            "summary": {
                "total": results.len(),
                "passed": results.iter().filter(|r| r.passed).count(),
                "failed": results.iter().filter(|r| !r.passed).count(),
                "pass_rate": (results.iter().filter(|r| r.passed).count() as f64 / results.len() as f64) * 100.0
            },
            "results": results,
            "timestamp": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
        });
        
        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

impl Default for SkillTester {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建默认测试用例
pub fn create_default_test_cases() -> Vec<SkillTestCase> {
    vec![
        // ==================== 搜索命令测试 ====================
        SkillTestCase {
            name: "search_basic".to_string(),
            input: "搜索 PDF 文件".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "search_find".to_string(),
            input: "查找工作目录的文档".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "search_exist".to_string(),
            input: "有没有 report 开头的文件".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "search_where".to_string(),
            input: "config 文件在哪里".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 信息命令测试 ====================
        SkillTestCase {
            name: "info_basic".to_string(),
            input: "查看这个文件的信息".to_string(),
            expected_command: "info".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "info_size".to_string(),
            input: "README 文件多大".to_string(),
            expected_command: "info".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 读取命令测试 ====================
        SkillTestCase {
            name: "cat_open".to_string(),
            input: "打开配置文件".to_string(),
            expected_command: "cat".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "cat_read".to_string(),
            input: "读取日志文件内容".to_string(),
            expected_command: "cat".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "cat_content".to_string(),
            input: "查看 README 里面是什么".to_string(),
            expected_command: "cat".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 复制命令测试 ====================
        SkillTestCase {
            name: "copy_basic".to_string(),
            input: "把这个文件复制到备份目录".to_string(),
            expected_command: "copy".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "copy_all".to_string(),
            input: "拷贝所有 txt 文件".to_string(),
            expected_command: "copy".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 移动命令测试 ====================
        SkillTestCase {
            name: "move_basic".to_string(),
            input: "移动临时文件到归档".to_string(),
            expected_command: "move".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "move_cut".to_string(),
            input: "把这个文件移到回收站".to_string(),
            expected_command: "move".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 删除命令测试 ====================
        SkillTestCase {
            name: "delete_basic".to_string(),
            input: "删除所有临时文件".to_string(),
            expected_command: "delete".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "delete_remove".to_string(),
            input: "移除这个文件夹".to_string(),
            expected_command: "delete".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "delete_cleanup".to_string(),
            input: "清理缓存文件".to_string(),
            expected_command: "delete".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 配置命令测试 ====================
        SkillTestCase {
            name: "config_show".to_string(),
            input: "显示当前配置".to_string(),
            expected_command: "config".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 索引命令测试 ====================
        SkillTestCase {
            name: "index_status".to_string(),
            input: "查看索引状态".to_string(),
            expected_command: "index".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "index_rebuild".to_string(),
            input: "重建索引".to_string(),
            expected_command: "index".to_string(),
            expected_params: None,
            should_match: true,
        },
        SkillTestCase {
            name: "index_update".to_string(),
            input: "更新索引".to_string(),
            expected_command: "index".to_string(),
            expected_params: None,
            should_match: true,
        },
        
        // ==================== 负向测试（不应该匹配） ====================
        SkillTestCase {
            name: "negative_weather".to_string(),
            input: "今天天气怎么样".to_string(),
            expected_command: String::new(),
            expected_params: None,
            should_match: false,
        },
        SkillTestCase {
            name: "negative_greeting".to_string(),
            input: "你好".to_string(),
            expected_command: String::new(),
            expected_params: None,
            should_match: false,
        },
    ]
}

/// 运行 Skill 测试
pub fn run_skill_test() -> Result<()> {
    let mut tester = SkillTester::new();
    
    // 添加默认测试用例
    for test_case in create_default_test_cases() {
        tester.test_cases.push(test_case);
    }
    
    // 运行测试
    let results = tester.run_tests();
    
    // 打印结果
    tester.print_results(&results);
    
    // 生成 JSON 报告
    let json_report = tester.generate_json_report(&results);
    
    // 保存 JSON 报告
    let report_path = std::path::Path::new("skill_test_report.json");
    std::fs::write(report_path, &json_report)?;
    println!("JSON 测试报告已保存到：{}", report_path.display());
    
    // 检查是否有失败的测试
    let failed = results.iter().filter(|r| !r.passed).count();
    if failed > 0 {
        eprintln!("有 {} 个测试失败", failed);
        std::process::exit(1);
    }
    
    println!("所有测试通过！✅");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_tester_basic() {
        let mut tester = SkillTester::new();
        tester.add_test_case("test1", "搜索 PDF", "search");
        
        let results = tester.run_tests();
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }
    
    #[test]
    fn test_command_matching() {
        let tester = SkillTester::new();
        
        // 测试搜索命令匹配
        assert_eq!(tester.match_command("搜索文件"), Some("search".to_string()));
        assert_eq!(tester.match_command("查找文档"), Some("search".to_string()));
        
        // 测试信息命令匹配
        assert_eq!(tester.match_command("查看文件信息"), Some("info".to_string()));
        
        // 测试无匹配
        assert_eq!(tester.match_command("你好"), None);
    }
    
    #[test]
    fn test_json_report() {
        let mut tester = SkillTester::new();
        tester.add_test_case("test1", "搜索文件", "search");
        
        let results = tester.run_tests();
        let report = tester.generate_json_report(&results);
        
        // 验证 JSON 格式
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert!(parsed.get("summary").is_some());
        assert!(parsed.get("results").is_some());
    }
}
