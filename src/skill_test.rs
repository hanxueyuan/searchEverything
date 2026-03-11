/// OpenClaw Skill 测试工具
/// 
/// 用于测试 Skill 触发和命令生成

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Skill 测试用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTestCase {
    /// 用户输入
    pub input: String,
    /// 期望触发的命令
    pub expected_command: String,
    /// 期望的参数
    pub expected_params: Option<serde_json::Value>,
}

/// Skill 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTestResult {
    /// 测试用例
    pub test_case: SkillTestCase,
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
    pub fn add_test_case(&mut self, input: &str, expected_command: &str) {
        self.test_cases.push(SkillTestCase {
            input: input.to_string(),
            expected_command: expected_command.to_string(),
            expected_params: None,
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
        // TODO: 实际测试 OpenClaw Skill 触发
        // 这里先实现一个简单的匹配逻辑
        
        let input_lower = test_case.input.to_lowercase();
        
        // 简单的关键词匹配
        let matched_command = if input_lower.contains("搜索") || input_lower.contains("查找") || input_lower.contains("有没有") {
            Some("search".to_string())
        } else if input_lower.contains("信息") || input_lower.contains("详情") || input_lower.contains("多大") {
            Some("info".to_string())
        } else if input_lower.contains("读取") || input_lower.contains("打开") {
            Some("cat".to_string())
        } else if input_lower.contains("复制") || input_lower.contains("拷贝") {
            Some("copy".to_string())
        } else if input_lower.contains("移动") || input_lower.contains("剪切") {
            Some("move".to_string())
        } else if input_lower.contains("删除") || input_lower.contains("移除") {
            Some("delete".to_string())
        } else if input_lower.contains("配置") {
            Some("config".to_string())
        } else if input_lower.contains("索引") {
            Some("index".to_string())
        } else {
            None
        };
        
        let passed = matched_command.as_ref() == Some(&test_case.expected_command);
        
        let error = if !passed {
            Some(format!(
                "期望命令：{}, 实际命令：{:?}",
                test_case.expected_command,
                matched_command
            ))
        } else {
            None
        };
        
        SkillTestResult {
            test_case: test_case.clone(),
            passed,
            actual_command: matched_command,
            actual_params: None,
            error,
        }
    }
    
    /// 打印测试结果
    pub fn print_results(&self, results: &[SkillTestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        println!("\n========================================");
        println!("Skill 测试结果");
        println!("========================================");
        println!("总计：{} | 通过：{} | 失败：{}", total, passed, failed);
        println!("通过率：{:.1}%", (passed as f64 / total as f64) * 100.0);
        println!();
        
        for (i, result) in results.iter().enumerate() {
            let status = if result.passed { "✅" } else { "❌" };
            println!("{}. {} {}", i + 1, status, result.test_case.input);
            
            if !result.passed {
                if let Some(ref error) = result.error {
                    println!("   错误：{}", error);
                }
            }
        }
        
        println!();
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
        // 搜索命令
        SkillTestCase {
            input: "搜索 PDF 文件".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
        },
        SkillTestCase {
            input: "查找工作目录的文档".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
        },
        SkillTestCase {
            input: "有没有 report 开头的文件".to_string(),
            expected_command: "search".to_string(),
            expected_params: None,
        },
        
        // 信息命令
        SkillTestCase {
            input: "查看这个文件的信息".to_string(),
            expected_command: "info".to_string(),
            expected_params: None,
        },
        SkillTestCase {
            input: "README 文件多大".to_string(),
            expected_command: "info".to_string(),
            expected_params: None,
        },
        
        // 读取命令
        SkillTestCase {
            input: "打开配置文件".to_string(),
            expected_command: "cat".to_string(),
            expected_params: None,
        },
        SkillTestCase {
            input: "读取日志文件内容".to_string(),
            expected_command: "cat".to_string(),
            expected_params: None,
        },
        
        // 复制命令
        SkillTestCase {
            input: "把这个文件复制到备份目录".to_string(),
            expected_command: "copy".to_string(),
            expected_params: None,
        },
        SkillTestCase {
            input: "拷贝所有 txt 文件".to_string(),
            expected_command: "copy".to_string(),
            expected_params: None,
        },
        
        // 移动命令
        SkillTestCase {
            input: "移动临时文件到归档".to_string(),
            expected_command: "move".to_string(),
            expected_params: None,
        },
        
        // 删除命令
        SkillTestCase {
            input: "删除所有临时文件".to_string(),
            expected_command: "delete".to_string(),
            expected_params: None,
        },
        
        // 配置命令
        SkillTestCase {
            input: "显示当前配置".to_string(),
            expected_command: "config".to_string(),
            expected_params: None,
        },
        
        // 索引命令
        SkillTestCase {
            input: "查看索引状态".to_string(),
            expected_command: "index".to_string(),
            expected_params: None,
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
    
    // 检查是否有失败的测试
    let failed = results.iter().filter(|r| !r.passed).count();
    if failed > 0 {
        eprintln!("有 {} 个测试失败", failed);
        std::process::exit(1);
    }
    
    println!("所有测试通过！✅");
    Ok(())
}
