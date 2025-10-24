# 设计文档

## 概述

Move 函数分析器是一个基于现有 Sui Move 分析器的扩展功能，它能够分析 Sui Move 项目中的函数，提取函数的详细信息包括源代码、参数、位置和调用关系。该工具将利用现有的 Move 编译器和 AST 解析功能来实现准确的代码分析。

## 架构

### 核心组件

```
┌─────────────────────────────────────────────────────────────┐
│                    Move Function Analyzer                   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Project Loader │  │ Function Parser │  │ Call Analyzer│ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ AST Processor   │  │ Type Resolver   │  │ JSON Formatter│ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                   Existing Move Infrastructure               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │    Project      │  │ ProjectContext  │  │   Symbols    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 数据流

1. **项目导入** → Project Loader → Project 结构
2. **函数搜索** → Function Parser → 函数 AST 节点
3. **代码提取** → AST Processor → 源代码和位置信息
4. **参数解析** → Type Resolver → 参数类型信息
5. **调用分析** → Call Analyzer → 函数调用关系
6. **结果格式化** → JSON Formatter → 最终 JSON 输出

## 组件和接口

### 1. FunctionAnalyzer (主要接口)

```rust
pub struct FunctionAnalyzer {
    project: Project,
    context: ProjectContext,
}

impl FunctionAnalyzer {
    pub fn new(project_path: PathBuf) -> Result<Self>;
    pub fn analyze_function(&self, function_name: &str) -> Result<Vec<FunctionAnalysis>>;
}
```

### 2. FunctionAnalysis (结果数据结构)

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionAnalysis {
    pub contract: String,           // 模块名称
    pub function: String,           // 函数签名
    pub source: String,             // 源代码
    pub location: LocationInfo,     // 位置信息
    pub parameters: Vec<Parameter>, // 参数列表
    pub calls: Vec<FunctionCall>,   // 函数调用
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocationInfo {
    pub file: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Parameter {
    pub name: String,
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionCall {
    pub file: PathBuf,
    pub function: String,
    pub module: String,
}
```

### 3. ProjectLoader

```rust
pub struct ProjectLoader;

impl ProjectLoader {
    pub fn load_project(path: PathBuf) -> Result<Project>;
    fn validate_move_project(path: &Path) -> Result<()>;
    fn parse_move_toml(path: &Path) -> Result<SourceManifest>;
}
```

### 4. FunctionParser

```rust
pub struct FunctionParser<'a> {
    project: &'a Project,
    context: &'a ProjectContext,
}

impl<'a> FunctionParser<'a> {
    pub fn find_functions(&self, name: &str) -> Vec<FunctionDef>;
    fn search_in_module(&self, module: &ModuleDef, name: &str) -> Vec<FunctionDef>;
    fn extract_function_signature(&self, func: &Function) -> String;
}
```

### 5. CallAnalyzer

```rust
pub struct CallAnalyzer<'a> {
    project: &'a Project,
    context: &'a ProjectContext,
}

impl<'a> CallAnalyzer<'a> {
    pub fn analyze_calls(&self, function: &Function) -> Vec<FunctionCall>;
    fn extract_function_calls(&self, exp: &Exp) -> Vec<FunctionCall>;
    fn resolve_call_target(&self, call: &ExpCall) -> Option<FunctionCall>;
}
```

## 数据模型

### 内部数据结构

```rust
// 扩展现有的 Function 结构
pub struct FunctionDef {
    pub function: Function,          // 来自 AST 的函数定义
    pub module_info: ModuleInfo,     // 所属模块信息
    pub location: Loc,               // 源码位置
}

pub struct ModuleInfo {
    pub address: AccountAddress,
    pub name: Symbol,
    pub file_path: PathBuf,
}
```

### JSON 输出格式

基于需求中的示例，输出格式为：

```json
{
    "contract": "ModuleName",
    "function": "function_name(param1: Type1, param2: Type2): ReturnType",
    "source": "    public fun function_name(param1: Type1, param2: Type2): ReturnType {\n        // function body\n    }",
    "location": {
        "file": "/path/to/file.move",
        "start_line": 10,
        "end_line": 15
    },
    "parameters": [
        {
            "name": "param1",
            "type": "Type1"
        },
        {
            "name": "param2", 
            "type": "Type2"
        }
    ],
    "calls": [
        {
            "file": "/path/to/other_file.move",
            "function": "other_function(Type3): Type4",
            "module": "OtherModule"
        }
    ]
}
```

## 错误处理

### 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("项目路径无效: {0}")]
    InvalidProjectPath(PathBuf),
    
    #[error("Move.toml 文件不存在或无效")]
    InvalidMoveToml,
    
    #[error("函数未找到: {0}")]
    FunctionNotFound(String),
    
    #[error("解析错误: {0}")]
    ParseError(String),
    
    #[error("类型解析错误: {0}")]
    TypeResolutionError(String),
}
```

### 错误处理策略

1. **项目加载错误**: 返回详细的错误信息，指导用户修复项目结构
2. **函数未找到**: 返回空结果数组，不抛出异常
3. **解析错误**: 记录错误日志，跳过有问题的函数，继续处理其他函数
4. **类型解析错误**: 使用 "unknown" 作为类型占位符，不中断分析过程

## 测试策略

### 单元测试

1. **ProjectLoader 测试**
   - 测试有效 Move 项目的加载
   - 测试无效项目路径的错误处理
   - 测试 Move.toml 解析

2. **FunctionParser 测试**
   - 测试函数查找功能
   - 测试函数签名提取
   - 测试多个同名函数的处理

3. **CallAnalyzer 测试**
   - 测试函数调用识别
   - 测试方法调用语法处理
   - 测试外部模块调用解析

4. **JSON 格式化测试**
   - 测试输出格式的正确性
   - 测试特殊字符的转义
   - 测试大型函数的处理

### 集成测试

1. **完整工作流测试**
   - 使用真实的 Sui Move 项目进行端到端测试
   - 测试复杂的函数调用关系
   - 测试泛型函数的处理

2. **性能测试**
   - 测试大型项目的分析性能
   - 测试内存使用情况
   - 测试并发分析能力

### 测试数据

使用现有的测试项目：
- `tests/beta_2024/project1`
- `tests/beta_2024/project2`
- `tests/alpha_2024/nft-protocol`

## 实现细节

### Move 语法特性支持

1. **函数定义类型**
   - `public fun` - 公共函数
   - `public(friend) fun` - 友元函数
   - `fun` - 私有函数
   - `entry fun` - 入口函数
   - `native fun` - 原生函数

2. **参数类型处理**
   - 基本类型: `u8`, `u64`, `u128`, `bool`, `address`
   - 引用类型: `&T`, `&mut T`
   - 泛型类型: `T`, `T: copy + drop`
   - 结构体类型: `ModuleName::StructName<T>`

3. **函数调用识别**
   - 直接调用: `function_name(args)`
   - 方法调用: `object.method(args)`
   - 模块调用: `module::function(args)`
   - 完全限定调用: `address::module::function(args)`

### 与现有代码集成

1. **利用现有的 Project 结构**
   - 复用项目加载和解析逻辑
   - 利用现有的 AST 处理功能
   - 使用现有的类型解析系统

2. **扩展现有功能**
   - 在现有的符号表基础上添加函数分析
   - 利用现有的位置映射功能
   - 复用现有的错误处理机制

3. **保持兼容性**
   - 不修改现有的核心结构
   - 通过新的模块添加功能
   - 保持现有 API 的稳定性