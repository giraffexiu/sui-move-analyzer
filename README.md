# Sui Move Analyzer | Sui Move 分析器

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.7%2B-blue.svg)](https://www.python.org/)

[English](#english) | [中文](#chinese)

---

## English

**Sui Move Analyzer** is a powerful toolkit for analyzing Move functions in Sui Move projects, developed by [MoveBit](https://movebit.xyz). It provides deep function analysis capabilities with both command-line interface and Python bindings for programmatic access.

### 🚀 Features

- **Deep Function Analysis**: Extract source code, analyze parameters, and generate call graphs
- **Python Library**: Easy-to-use Python API for programmatic function analysis
- **Command Line Tool**: Standalone binary for function analysis in CI/CD pipelines
- **Comprehensive Results**: Function signatures, source code, location info, parameters, and call relationships
- **Multi-project Support**: Works with Sui Move projects and supports various Move language features
- **Type Safety**: Full type hints for better IDE support
- **Error Handling**: Comprehensive exception handling with specific error types

### 📦 Installation

#### Option 1: Install Python Package (Recommended)

```bash
pip install sui-move-analyzer
```

This will automatically download and install the pre-built binary along with the Python library.

#### Option 2: Build from Source

**Prerequisites:**
- Rust toolchain (1.70.0 or later)
- Cargo

**Steps:**

```bash
# Clone the repository
git clone https://github.com/movebit/sui-move-analyzer.git
cd sui-move-analyzer

# Build the Rust binary
cd src/beta-2024
cargo build --release
cd ../..

# The binary will be available at:
# src/beta-2024/target/release/move-function-analyzer

# (Optional) Install Python package from source
pip install -e .
```

**Verify Installation:**

```bash
# Test Python import
python -c "from sui_move_analyzer import MoveFunctionAnalyzer; print('✓ Installation successful!')"

# Test with a sample project
./src/beta-2024/target/release/move-function-analyzer ./tests/deepbook modify_order
```

### 🔧 Usage

#### Python API

**Basic Usage:**

```python
from sui_move_analyzer import MoveFunctionAnalyzer

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

**Error Handling:**

```python
from sui_move_analyzer import (
    MoveFunctionAnalyzer,
    ProjectNotFoundError,
    FunctionNotFoundError,
    AnalysisFailedError
)

analyzer = MoveFunctionAnalyzer()

try:
    results = analyzer.analyze("./my-project", "transfer")
    for result in results:
        print(f"Found in module: {result.contract}")
        print(f"Parameters: {len(result.parameters)}")
except ProjectNotFoundError as e:
    print(f"Project not found: {e.project_path}")
except FunctionNotFoundError as e:
    print(f"Function '{e.function_name}' not found")
except AnalysisFailedError as e:
    print(f"Analysis failed: {e}")
```

**Convenience Function:**

```python
from sui_move_analyzer import analyze_function

# Quick analysis without creating an analyzer instance
results = analyze_function("./my-project", "mint")
```

#### Command Line Tool

**Binary Location:**

If you built from source, the binary is located at:
```bash
./src/beta-2024/target/release/move-function-analyzer
```

**Usage:**
```bash
# Analyze a specific function
move-function-analyzer <project_path> <function_name>

# Example
./src/beta-2024/target/release/move-function-analyzer ./tests/deepbook modify_order

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

#### Python API

##### `MoveFunctionAnalyzer`

Main analyzer class for function analysis.

**Constructor:**
```python
analyzer = MoveFunctionAnalyzer(binary_path: Optional[str] = None)
```
- `binary_path`: Optional path to the analyzer binary. If not provided, uses the bundled binary.

**Methods:**

**`analyze(project_path, function_name) → List[AnalysisResult]`**

Analyze a Move function and return structured results.

- **Parameters:**
  - `project_path` (str | Path): Path to the Move project directory (containing Move.toml)
  - `function_name` (str): Name of the function to analyze
- **Returns:** List of `AnalysisResult` objects
- **Raises:**
  - `ProjectNotFoundError`: If the project path doesn't exist
  - `FunctionNotFoundError`: If the function is not found
  - `AnalysisFailedError`: If the analysis process fails

**`analyze_raw(project_path, function_name) → Dict[str, Any]`**

Analyze a Move function and return raw JSON data.

- **Parameters:** Same as `analyze()`
- **Returns:** Raw JSON data as dictionary

##### Data Classes

**`AnalysisResult`**

Contains complete function analysis information.

- `contract: str` - Module name containing the function
- `function: str` - Function signature with parameters and return type
- `source: str` - Complete source code of the function
- `location: LocationInfo` - File location information
- `parameters: List[Parameter]` - Function parameters
- `calls: List[FunctionCall]` - Functions called by this function

**`LocationInfo`**

File location information.

- `file: str` - Path to the source file
- `start_line: int` - Starting line number (1-indexed)
- `end_line: int` - Ending line number (1-indexed)

**`Parameter`**

Function parameter information.

- `name: str` - Parameter name
- `type: str` - Parameter type as string

**`FunctionCall`**

Information about function calls.

- `file: str` - File containing the called function
- `function: str` - Function signature of the called function
- `module: str` - Module name containing the called function

##### Exceptions

- `AnalyzerError` - Base exception class
- `ProjectNotFoundError` - Project path doesn't exist or is invalid
- `FunctionNotFoundError` - Function not found in the project
- `BinaryNotFoundError` - Analyzer binary not found
- `AnalysisFailedError` - Analysis process failed

##### Convenience Functions

**`analyze_function(project_path, function_name) → List[AnalysisResult]`**

Quick analysis without creating an analyzer instance.

```python
from sui_move_analyzer import analyze_function
results = analyze_function("./my-project", "mint")
```

### 🔍 Examples

#### Example 1: Basic Function Analysis

```python
from sui_move_analyzer import MoveFunctionAnalyzer

analyzer = MoveFunctionAnalyzer()
results = analyzer.analyze("./tests/deepbook", "modify_order")

for result in results:
    print(f"Module: {result.contract}")
    print(f"Function: {result.function}")
    print(f"Location: {result.location.file}:{result.location.start_line}-{result.location.end_line}")
    print(f"Parameters: {len(result.parameters)}")
    for param in result.parameters:
        print(f"  - {param.name}: {param.type}")
```

#### Example 2: Command Line Analysis

```bash
# Analyze the modify_order function in the deepbook project
./src/beta-2024/target/release/move-function-analyzer ./tests/deepbook modify_order
```

**Output:**
```json
[
  {
    "contract": "book",
    "function": "public(package) fun modify_order(self: &mut Book, order_id: u128, new_quantity: u64, timestamp: u64): (u64, &Order)",
    "source": "\n    /// Modifies an order given order_id and new_quantity.\n    /// New quantity must be less than the original quantity.\n    /// Order must not have already expired.\n    public(package) fun modify_order(self: &mut Book, order_id: u128, new_quantity: u64, timestamp: u64): (u64, &Order) {\n        assert!(new_quantity >= self.min_size, EOrderBelowMinimumSize);\n        assert!(new_quantity % self.lot_size == 0, EOrderInvalidLotSize);\n\n        let order = self.book_side(order_id).borrow_mut(order_id);\n        assert!(new_quantity < order.quantity(), ENewQuantityMustBeLessThanOriginal);\n        let cancel_quantity = order.quantity() - new_quantity;\n        order.modify(new_quantity, timestamp);\n\n        (cancel_quantity, order)\n    }",
    "location": {
      "file": "/path/to/sources/book/book.move",
      "start_line": 154,
      "end_line": 164
    },
    "parameter": [
      {"name": "self", "type": "&mut Book"},
      {"name": "order_id", "type": "u128"},
      {"name": "new_quantity", "type": "u64"},
      {"name": "timestamp", "type": "u64"}
    ],
    "calls": []
  }
]
```

#### Example 3: Analyzing Multiple Functions

```python
from sui_move_analyzer import MoveFunctionAnalyzer

analyzer = MoveFunctionAnalyzer()

# Analyze multiple functions
functions = ["mint", "transfer", "burn"]
for func_name in functions:
    try:
        results = analyzer.analyze("./my-nft-project", func_name)
        print(f"\n{func_name}: Found {len(results)} implementation(s)")
        for result in results:
            print(f"  - {result.contract}::{func_name}")
    except Exception as e:
        print(f"{func_name}: {e}")
```

#### Example 4: Working with Raw JSON

```python
from sui_move_analyzer import MoveFunctionAnalyzer
import json

analyzer = MoveFunctionAnalyzer()
raw_data = analyzer.analyze_raw("./my-project", "transfer")

# Pretty print the JSON
print(json.dumps(raw_data, indent=2))

# Save to file
with open("analysis_result.json", "w") as f:
    json.dump(raw_data, f, indent=2)
```

See the [tests](./tests/) directory for more Move project examples.

### 🐛 Troubleshooting

#### Binary not found

If you get a "Binary not found" error:

1. Make sure you built the Rust binary:
   ```bash
   cd src/beta-2024
   cargo build --release
   ```

2. Check that the binary exists:
   ```bash
   ls -lh src/beta-2024/target/release/move-function-analyzer
   ```

3. Reinstall the Python package:
   ```bash
   pip install -e . --force-reinstall
   ```

#### Permission denied

On Unix systems, make sure the binary is executable:

```bash
chmod +x src/beta-2024/target/release/move-function-analyzer
```

#### Import error

If you get an import error:

1. Check that the package is installed:
   ```bash
   pip list | grep sui-move-analyzer
   ```

2. Reinstall if necessary:
   ```bash
   pip uninstall sui-move-analyzer
   pip install sui-move-analyzer
   ```

#### Function not found

If the analyzer can't find your function:

1. Make sure the function name is spelled correctly (case-sensitive)
2. Verify the function exists in your Move source files
3. Check that your Move.toml is valid
4. Ensure all dependencies are properly configured

### 📚 Project Structure

```
sui-move-analyzer/
├── src/
│   └── beta-2024/           # Rust analyzer implementation
│       ├── src/
│       │   ├── function_analyzer.rs  # Core analysis logic
│       │   └── ...
│       ├── Cargo.toml
│       └── target/release/
│           └── move-function-analyzer  # Binary
├── sui_move_analyzer/       # Python package
│   ├── __init__.py
│   ├── analyzer.py          # Main Python API
│   ├── exceptions.py        # Exception classes
│   └── bin/                 # Binary location after install
├── setup.py                 # Python package configuration
├── install.py               # Installation script
├── test.py                  # Test script
├── tests/                   # Test Move projects
│   ├── deepbook/
│   └── ...
└── README.md               # This file
```

### 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

**Development Setup:**

```bash
# Clone the repository
git clone https://github.com/movebit/sui-move-analyzer.git
cd sui-move-analyzer

# Build the Rust binary
cd src/beta-2024
cargo build --release
cd ../..

# Install Python package in development mode
pip install -e .

# Run tests
python test_install.py
```

### 📄 License

This project is licensed under the Apache License 2.0.

### 🔗 Links

- **GitHub**: https://github.com/movebit/sui-move-analyzer
- **Issues**: https://github.com/movebit/sui-move-analyzer/issues
- **MoveBit**: https://movebit.xyz

---

## Chinese

**Sui Move 分析器**是由 [MoveBit](https://movebit.xyz) 开发的强大 Move 函数分析工具包，为 Sui Move 项目提供深度函数分析功能。它提供命令行界面和 Python 绑定，支持程序化访问。

### 🚀 功能特性

- **深度函数分析**：提取源码、分析参数、生成调用图
- **Python 库**：易于使用的 Python API，支持程序化函数分析
- **命令行工具**：独立的二进制工具，适用于 CI/CD 流水线中的函数分析
- **全面的结果**：函数签名、源代码、位置信息、参数和调用关系
- **多项目支持**：支持 Sui Move 项目和各种 Move 语言特性

### 📦 安装方式

#### 快速安装

```bash
pip install sui-move-analyzer
```

#### 从源码构建

```bash
git clone https://github.com/movebit/sui-move-analyzer.git
cd sui-move-analyzer
cd src/beta-2024
cargo build --release
```

详细的安装说明、故障排除和验证步骤，请参阅 [INSTALL.md](./INSTALL.md)。

### 🔧 使用方法

#### Python API

```python
from sui_move_analyzer import MoveFunctionAnalyzer

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

如果从源码构建，二进制文件位于：
```bash
./src/beta-2024/target/release/move-function-analyzer
```

**使用方法：**
```bash
# 分析特定函数
move-function-analyzer <项目路径> <函数名>

# 示例
move-function-analyzer ./tests/deepbook modify_order

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

#### 分析函数

```bash
# 分析 deepbook 项目中的 modify_order 函数
./src/beta-2024/target/release/move-function-analyzer ./tests/deepbook modify_order
```

**输出：**
```json
[
  {
    "contract": "book",
    "function": "public(package) fun modify_order(self: &mut Book, order_id: u128, new_quantity: u64, timestamp: u64): (u64, &Order)",
    "source": "...",
    "location": {
      "file": "/path/to/sources/book/book.move",
      "start_line": 154,
      "end_line": 164
    },
    "parameter": [
      {"name": "self", "type": "&mut Book"},
      {"name": "order_id", "type": "u128"},
      {"name": "new_quantity", "type": "u64"},
      {"name": "timestamp", "type": "u64"}
    ],
    "calls": []
  }
]
```

查看 [tests](./tests/) 目录获取更多 Move 项目示例。

### 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

### 📄 许可证

本项目采用 Apache License 2.0 许可证。
