# Move Function Analyzer | Move 函数分析器

[English](#english) | [中文](#chinese)

---

## English

**Move Function Analyzer** is a powerful toolkit for analyzing Move functions in Sui Move projects, developed by [MoveBit](https://movebit.xyz). It provides deep function analysis capabilities with both command-line interface and Python bindings for programmatic access.

### 🚀 Features

- **Deep Function Analysis**: Extract source code, analyze parameters, and generate call graphs
- **Python Library**: Easy-to-use Python API for programmatic function analysis
- **Command Line Tool**: Standalone binary for function analysis in CI/CD pipelines
- **Comprehensive Results**: Function signatures, source code, location info, parameters, and call relationships
- **Multi-project Support**: Works with Sui Move projects and supports various Move language features

### 📦 Installation

#### Python Library

```bash
pip install move-function-analyzer
```

#### Build from Source

```bash
git clone https://github.com/movebit/sui-move-analyzer.git
cd sui-move-analyzer
cargo build --release
```

The binary will be available at `target/release/move-function-analyzer`.

### 🔧 Usage

#### Python API

```python
from move_function_analyzer import MoveFunctionAnalyzer

# Create analyzer instance
analyzer = MoveFunctionAnalyzer()

# Analyze a function
results = analyzer.analyze("/path/to/move/project", "transfer")

# Process results
for result in results:
    print(f"Contract: {result.contract}")
    print(f"Function: {result.function}")
    print(f"Source Code:\n{result.source}")
    print(f"Parameters: {[p.name + ': ' + p.type for p in result.parameters]}")
    print(f"Function Calls: {len(result.calls)}")
    print("---")
```

#### Command Line Tool

```bash
# Analyze a specific function
move-function-analyzer /path/to/project function_name

# Example output (JSON format)
[
  {
    "contract": "my_module::nft",
    "function": "mint(name: vector<u8>, ctx: &mut TxContext): NFT",
    "source": "public fun mint(name: vector<u8>, ctx: &mut TxContext): NFT {\n    let nft = NFT {\n        id: object::new(ctx),\n        name,\n    };\n    nft\n}",
    "location": {
      "file": "/path/to/sources/nft.move",
      "start_line": 25,
      "end_line": 32
    },
    "parameter": [
      {"name": "name", "type": "vector<u8>"},
      {"name": "ctx", "type": "&mut TxContext"}
    ],
    "calls": [
      {
        "file": "/path/to/sources/nft.move",
        "function": "new(ctx: &mut TxContext): UID",
        "module": "sui::object"
      }
    ]
  }
]
```

### 📊 Analysis Results

The analyzer provides comprehensive information about Move functions:

- **Function Signature**: Complete signature with parameters and return types
- **Source Code**: Full function implementation
- **Location Info**: File path and line numbers
- **Parameters**: Detailed parameter information with types
- **Call Graph**: Functions called within the analyzed function
- **Module Context**: Module and contract information

### 🛠️ API Reference

#### Python Classes

##### `MoveFunctionAnalyzer`
Main analyzer class for function analysis.

**Methods:**
- `analyze(project_path: str, function_name: str) → List[AnalysisResult]`
  - Analyzes functions and returns structured results
- `analyze_raw(project_path: str, function_name: str) → Dict[str, Any]`
  - Returns raw JSON analysis results

##### `AnalysisResult`
Contains complete function analysis information.

**Attributes:**
- `contract: str` - Module name
- `function: str` - Function signature  
- `source: str` - Source code
- `location: LocationInfo` - File location
- `parameters: List[Parameter]` - Function parameters
- `calls: List[FunctionCall]` - Function calls

##### `LocationInfo`
File location information.

**Attributes:**
- `file: str` - File path
- `start_line: int` - Start line number
- `end_line: int` - End line number

##### `Parameter`
Function parameter information.

**Attributes:**
- `name: str` - Parameter name
- `type: str` - Parameter type

##### `FunctionCall`
Information about function calls.

**Attributes:**
- `file: str` - File containing the called function
- `function: str` - Called function signature
- `module: str` - Module containing the called function

### 🔍 Examples

See the [examples](./examples/) directory for complete usage examples including:
- Basic NFT project analysis
- Complex function call analysis
- Marketplace contract analysis

### 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### 📄 License

This project is licensed under the Apache License 2.0.

---

## Chinese

**Move 函数分析器**是由 [MoveBit](https://movebit.xyz) 开发的强大 Move 函数分析工具包，为 Sui Move 项目提供深度函数分析功能。它提供命令行界面和 Python 绑定，支持程序化访问。

### 🚀 功能特性

- **深度函数分析**：提取源码、分析参数、生成调用图
- **Python 库**：易于使用的 Python API，支持程序化函数分析
- **命令行工具**：独立的二进制工具，适用于 CI/CD 流水线中的函数分析
- **全面的结果**：函数签名、源代码、位置信息、参数和调用关系
- **多项目支持**：支持 Sui Move 项目和各种 Move 语言特性

### 📦 安装方式

#### Python 库

```bash
pip install move-function-analyzer
```

#### 从源码构建

```bash
git clone git@github.com:giraffexiu/sui-move-analyzer.git
cd sui-move-analyzer
cargo build --release
```

二进制文件将位于 `target/release/move-function-analyzer`。

### 🔧 使用方法

#### Python API

```python
from move_function_analyzer import MoveFunctionAnalyzer

# 创建分析器实例
analyzer = MoveFunctionAnalyzer()

# 分析函数
results = analyzer.analyze("/path/to/move/project", "transfer")

# 处理结果
for result in results:
    print(f"合约: {result.contract}")
    print(f"函数: {result.function}")
    print(f"源代码:\n{result.source}")
    print(f"参数: {[p.name + ': ' + p.type for p in result.parameters]}")
    print(f"函数调用: {len(result.calls)}")
    print("---")
```

#### 命令行工具

```bash
# 分析特定函数
move-function-analyzer /path/to/project function_name

# 示例输出（JSON 格式）
[
  {
    "contract": "my_module::nft",
    "function": "mint(name: vector<u8>, ctx: &mut TxContext): NFT",
    "source": "public fun mint(name: vector<u8>, ctx: &mut TxContext): NFT {\n    let nft = NFT {\n        id: object::new(ctx),\n        name,\n    };\n    nft\n}",
    "location": {
      "file": "/path/to/sources/nft.move",
      "start_line": 25,
      "end_line": 32
    },
    "parameter": [
      {"name": "name", "type": "vector<u8>"},
      {"name": "ctx", "type": "&mut TxContext"}
    ],
    "calls": [
      {
        "file": "/path/to/sources/nft.move",
        "function": "new(ctx: &mut TxContext): UID",
        "module": "sui::object"
      }
    ]
  }
]
```

### 📊 分析结果

分析器提供 Move 函数的全面信息：

- **函数签名**：包含参数和返回类型的完整签名
- **源代码**：完整的函数实现
- **位置信息**：文件路径和行号
- **参数信息**：详细的参数信息和类型
- **调用图**：被分析函数内调用的其他函数
- **模块上下文**：模块和合约信息

### 🛠️ API 参考

#### Python 类

##### `MoveFunctionAnalyzer`
用于函数分析的主要分析器类。

**方法：**
- `analyze(project_path: str, function_name: str) → List[AnalysisResult]`
  - 分析函数并返回结构化结果
- `analyze_raw(project_path: str, function_name: str) → Dict[str, Any]`
  - 返回原始 JSON 分析结果

##### `AnalysisResult`
包含完整函数分析信息。

**属性：**
- `contract: str` - 模块名称
- `function: str` - 函数签名
- `source: str` - 源代码
- `location: LocationInfo` - 文件位置
- `parameters: List[Parameter]` - 函数参数
- `calls: List[FunctionCall]` - 函数调用

##### `LocationInfo`
文件位置信息。

**属性：**
- `file: str` - 文件路径
- `start_line: int` - 起始行号
- `end_line: int` - 结束行号

##### `Parameter`
函数参数信息。

**属性：**
- `name: str` - 参数名称
- `type: str` - 参数类型

##### `FunctionCall`
函数调用信息。

**属性：**
- `file: str` - 包含被调用函数的文件
- `function: str` - 被调用函数签名
- `module: str` - 包含被调用函数的模块

### 🔍 示例

查看 [examples](./examples/) 目录获取完整的使用示例，包括：
- 基础 NFT 项目分析
- 复杂函数调用分析
- 市场合约分析

### 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

### 📄 许可证

本项目采用 Apache License 2.0 许可证。
