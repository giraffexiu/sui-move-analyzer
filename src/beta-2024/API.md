# Move Function Analyzer API Documentation

This document provides comprehensive API documentation for the Move Function Analyzer library and command-line tool.

## Library API

### Core Types

#### `FunctionAnalysis`

The main result structure containing comprehensive function analysis information.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionAnalysis {
    /// The contract/module name containing the function
    pub contract: String,
    /// The function signature including name, parameters, and return type
    pub function: String,
    /// The complete source code of the function
    pub source: String,
    /// Location information including file path and line numbers
    pub location: LocationInfo,
    /// List of function parameters with names and types
    pub parameters: Vec<Parameter>,
    /// List of functions called by this function
    pub calls: Vec<FunctionCall>,
}
```

**Fields:**
- `contract`: Module name in the format `address::module_name`
- `function`: Complete function signature with parameters and return type
- `source`: Full source code with original formatting and indentation
- `location`: File location and line number information
- `parameters`: Array of parameter information
- `calls`: Array of function calls made within this function

#### `LocationInfo`

Location information for a function in the source code.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocationInfo {
    /// Path to the source file containing the function
    pub file: PathBuf,
    /// Starting line number of the function (1-indexed)
    pub start_line: u32,
    /// Ending line number of the function (1-indexed)
    pub end_line: u32,
}
```

**Fields:**
- `file`: Absolute path to the source file
- `start_line`: Starting line number (1-indexed, inclusive)
- `end_line`: Ending line number (1-indexed, inclusive)

#### `Parameter`

Information about a function parameter.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type as a string representation
    #[serde(rename = "type")]
    pub type_: String,
}
```

**Fields:**
- `name`: Parameter name as it appears in the function signature
- `type_`: Type string representation (e.g., `"u64"`, `"&mut vector<u8>"`, `"MyModule::MyStruct<T>"`)

#### `FunctionCall`

Information about a function call made within the analyzed function.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// Path to the file containing the called function
    pub file: PathBuf,
    /// Function signature of the called function
    pub function: String,
    /// Module name containing the called function
    pub module: String,
}
```

**Fields:**
- `file`: Path to the file containing the called function
- `function`: Signature of the called function
- `module`: Module name in the format `address::module_name`

### Error Types

#### `AnalyzerError`

Comprehensive error type for all analyzer operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("Invalid project path: {0}")]
    InvalidProjectPath(PathBuf),
    
    #[error("Move.toml file not found or invalid")]
    InvalidMoveToml,
    
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Type resolution error: {0}")]
    TypeResolutionError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}
```

**Variants:**
- `InvalidProjectPath`: The provided project path doesn't exist or isn't accessible
- `InvalidMoveToml`: Move.toml file is missing, unreadable, or contains invalid syntax
- `FunctionNotFound`: No functions with the specified name were found
- `ParseError`: Error parsing Move source code or project configuration
- `TypeResolutionError`: Error resolving complex types in function signatures
- `IoError`: File system I/O error
- `JsonError`: JSON serialization/deserialization error
- `AnalysisError`: General analysis error with descriptive message

### Main API

#### `FunctionAnalyzer`

The main analyzer interface for analyzing Move functions.

```rust
pub struct FunctionAnalyzer {
    project: Project,
    context: ProjectContext,
}
```

##### Methods

###### `new(project_path: PathBuf) -> AnalyzerResult<Self>`

Create a new function analyzer for the specified project.

**Parameters:**
- `project_path`: Path to the directory containing Move.toml

**Returns:**
- `AnalyzerResult<FunctionAnalyzer>`: New analyzer instance or error

**Errors:**
- `InvalidProjectPath`: Project path doesn't exist or isn't accessible
- `InvalidMoveToml`: Move.toml is missing or invalid
- `ParseError`: Error parsing project configuration
- `AnalysisError`: General project loading error

**Example:**
```rust
use beta_2024::function_analyzer::{FunctionAnalyzer, AnalyzerError};
use std::path::PathBuf;

fn create_analyzer() -> Result<FunctionAnalyzer, AnalyzerError> {
    let project_path = PathBuf::from("./my-move-project");
    FunctionAnalyzer::new(project_path)
}
```

###### `analyze_function(&self, function_name: &str) -> AnalyzerResult<Vec<FunctionAnalysis>>`

Analyze all functions with the specified name in the project.

**Parameters:**
- `function_name`: Name of the function to analyze

**Returns:**
- `AnalyzerResult<Vec<FunctionAnalysis>>`: Vector of analysis results (may be empty if no functions found)

**Errors:**
- `ParseError`: Error parsing function definitions
- `TypeResolutionError`: Error resolving parameter or return types
- `AnalysisError`: General analysis error

**Example:**
```rust
fn analyze_transfer_function(analyzer: &FunctionAnalyzer) -> Result<(), AnalyzerError> {
    let results = analyzer.analyze_function("transfer")?;
    
    for result in results {
        println!("Found function: {}", result.function);
        println!("Module: {}", result.contract);
        println!("Parameters: {}", result.parameters.len());
        println!("Calls: {}", result.calls.len());
        println!("Location: {}:{}-{}", 
                 result.location.file.display(),
                 result.location.start_line,
                 result.location.end_line);
    }
    
    Ok(())
}
```

### Utility Functions

#### `ProjectLoader`

Static utility for loading and validating Move projects.

##### Methods

###### `load_project(project_path: PathBuf) -> AnalyzerResult<Project>`

Load and validate a Move project.

**Parameters:**
- `project_path`: Path to the project directory

**Returns:**
- `AnalyzerResult<Project>`: Loaded project or error

**Example:**
```rust
use beta_2024::function_analyzer::ProjectLoader;
use std::path::PathBuf;

fn load_project() -> Result<(), Box<dyn std::error::Error>> {
    let project = ProjectLoader::load_project(PathBuf::from("./my-project"))?;
    println!("Project loaded successfully");
    Ok(())
}
```

## Command Line Interface

### Usage

```bash
move-function-analyzer [OPTIONS] --project-path <PATH> --function <NAME>
```

### Required Arguments

- `-p, --project-path <PATH>`: Path to the Sui Move project directory
- `-f, --function <NAME>`: Name of the function to analyze

### Optional Arguments

- `--pretty`: Format JSON output with indentation
- `-v, --verbose`: Enable verbose logging (can be repeated for more verbosity)
- `-q, --quiet`: Suppress non-error output
- `-h, --help`: Show help information
- `-V, --version`: Show version information

### Exit Codes

- `0`: Success
- `1`: Error (invalid arguments, project not found, analysis failed, etc.)

### Examples

#### Basic Usage

```bash
# Analyze a function in the current directory
move-function-analyzer -p . -f transfer

# Analyze with pretty-printed output
move-function-analyzer -p ./my-project -f mint --pretty

# Quiet mode (only JSON output)
move-function-analyzer -p ./my-project -f burn --quiet
```

#### Verbose Output

```bash
# Basic verbose output
move-function-analyzer -p . -f transfer -v

# Maximum verbosity for debugging
move-function-analyzer -p . -f transfer -vvv
```

#### Integration with Other Tools

```bash
# Save output to file
move-function-analyzer -p . -f transfer --pretty > transfer_analysis.json

# Extract function signatures
move-function-analyzer -p . -f transfer --quiet | jq -r '.[].function'

# Count parameters
move-function-analyzer -p . -f transfer --quiet | jq -r '.[] | .parameters | length'

# List called functions
move-function-analyzer -p . -f transfer --quiet | jq -r '.[] | .calls[].function'
```

## JSON Output Format

### Schema

The tool outputs JSON conforming to the following schema:

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "contract": {
        "type": "string",
        "description": "Module name containing the function"
      },
      "function": {
        "type": "string", 
        "description": "Complete function signature"
      },
      "source": {
        "type": "string",
        "description": "Complete source code of the function"
      },
      "location": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "description": "Path to the source file"
          },
          "start_line": {
            "type": "integer",
            "minimum": 1,
            "description": "Starting line number (1-indexed)"
          },
          "end_line": {
            "type": "integer", 
            "minimum": 1,
            "description": "Ending line number (1-indexed)"
          }
        },
        "required": ["file", "start_line", "end_line"]
      },
      "parameters": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "name": {
              "type": "string",
              "description": "Parameter name"
            },
            "type": {
              "type": "string",
              "description": "Parameter type"
            }
          },
          "required": ["name", "type"]
        }
      },
      "calls": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "file": {
              "type": "string",
              "description": "Path to file containing called function"
            },
            "function": {
              "type": "string",
              "description": "Signature of called function"
            },
            "module": {
              "type": "string",
              "description": "Module containing called function"
            }
          },
          "required": ["file", "function", "module"]
        }
      }
    },
    "required": ["contract", "function", "source", "location", "parameters", "calls"]
  }
}
```

### Example Output

```json
[
  {
    "contract": "simple_nft::nft",
    "function": "mint(name: vector<u8>, description: vector<u8>, ctx: &mut sui::tx_context::TxContext): simple_nft::nft::SimpleNFT",
    "source": "    public fun mint(\n        name: vector<u8>,\n        description: vector<u8>,\n        ctx: &mut TxContext\n    ): SimpleNFT {\n        let sender = tx_context::sender(ctx);\n        let nft = SimpleNFT {\n            id: object::new(ctx),\n            name: string::utf8(name),\n            description: string::utf8(description),\n            creator: sender,\n        };\n\n        sui::event::emit(NFTMinted {\n            nft_id: object::uid_to_address(&nft.id),\n            name: nft.name,\n            creator: sender,\n        });\n\n        nft\n    }",
    "location": {
      "file": "/path/to/project/sources/nft.move",
      "start_line": 25,
      "end_line": 45
    },
    "parameters": [
      {
        "name": "name",
        "type": "vector<u8>"
      },
      {
        "name": "description",
        "type": "vector<u8>"
      },
      {
        "name": "ctx",
        "type": "&mut sui::tx_context::TxContext"
      }
    ],
    "calls": [
      {
        "file": "/path/to/project/sources/nft.move",
        "function": "sender(ctx: &sui::tx_context::TxContext): address",
        "module": "sui::tx_context"
      },
      {
        "file": "/path/to/project/sources/nft.move",
        "function": "new(ctx: &mut sui::tx_context::TxContext): sui::object::UID",
        "module": "sui::object"
      },
      {
        "file": "/path/to/project/sources/nft.move",
        "function": "utf8(bytes: vector<u8>): std::string::String",
        "module": "std::string"
      }
    ]
  }
]
```

## Integration Examples

### Rust Integration

```rust
use beta_2024::function_analyzer::{FunctionAnalyzer, AnalyzerError};
use std::path::PathBuf;

fn main() -> Result<(), AnalyzerError> {
    // Create analyzer
    let analyzer = FunctionAnalyzer::new(PathBuf::from("./my-project"))?;
    
    // Analyze multiple functions
    let functions = vec!["mint", "transfer", "burn"];
    
    for func_name in functions {
        match analyzer.analyze_function(func_name) {
            Ok(results) => {
                println!("Function '{}': {} results", func_name, results.len());
                for result in results {
                    println!("  Module: {}", result.contract);
                    println!("  Parameters: {}", result.parameters.len());
                    println!("  Calls: {}", result.calls.len());
                }
            }
            Err(e) => {
                eprintln!("Error analyzing '{}': {}", func_name, e);
            }
        }
    }
    
    Ok(())
}
```

### Python Integration

```python
import subprocess
import json
from typing import List, Dict, Any, Optional

class MoveAnalyzer:
    def __init__(self, analyzer_path: str = "move-function-analyzer"):
        self.analyzer_path = analyzer_path
    
    def analyze_function(self, project_path: str, function_name: str) -> Optional[List[Dict[str, Any]]]:
        try:
            result = subprocess.run([
                self.analyzer_path,
                "--project-path", project_path,
                "--function", function_name,
                "--quiet"
            ], capture_output=True, text=True, check=True)
            
            return json.loads(result.stdout)
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            print(f"Error analyzing {function_name}: {e}")
            return None
    
    def get_function_complexity(self, project_path: str, function_name: str) -> Dict[str, int]:
        results = self.analyze_function(project_path, function_name)
        if not results:
            return {}
        
        complexity = {}
        for result in results:
            key = f"{result['contract']}::{function_name}"
            complexity[key] = {
                'parameters': len(result['parameters']),
                'calls': len(result['calls']),
                'lines': result['location']['end_line'] - result['location']['start_line'] + 1
            }
        
        return complexity

# Usage
analyzer = MoveAnalyzer()
results = analyzer.analyze_function("./my-project", "transfer")
if results:
    for result in results:
        print(f"Function: {result['function']}")
        print(f"Parameters: {len(result['parameters'])}")
```

### JavaScript/Node.js Integration

```javascript
const { spawn } = require('child_process');

class MoveAnalyzer {
    constructor(analyzerPath = 'move-function-analyzer') {
        this.analyzerPath = analyzerPath;
    }
    
    analyzeFunction(projectPath, functionName) {
        return new Promise((resolve, reject) => {
            const analyzer = spawn(this.analyzerPath, [
                '--project-path', projectPath,
                '--function', functionName,
                '--quiet'
            ]);
            
            let output = '';
            let error = '';
            
            analyzer.stdout.on('data', (data) => {
                output += data.toString();
            });
            
            analyzer.stderr.on('data', (data) => {
                error += data.toString();
            });
            
            analyzer.on('close', (code) => {
                if (code === 0) {
                    try {
                        resolve(JSON.parse(output));
                    } catch (e) {
                        reject(new Error(`Failed to parse JSON: ${e.message}`));
                    }
                } else {
                    reject(new Error(`Analysis failed: ${error}`));
                }
            });
        });
    }
    
    async getFunctionSignatures(projectPath, functionNames) {
        const signatures = [];
        
        for (const funcName of functionNames) {
            try {
                const results = await this.analyzeFunction(projectPath, funcName);
                for (const result of results) {
                    signatures.push({
                        name: funcName,
                        signature: result.function,
                        module: result.contract
                    });
                }
            } catch (error) {
                console.warn(`Failed to analyze ${funcName}: ${error.message}`);
            }
        }
        
        return signatures;
    }
}

// Usage
const analyzer = new MoveAnalyzer();
analyzer.analyzeFunction('./my-project', 'transfer')
    .then(results => {
        results.forEach(result => {
            console.log(`Function: ${result.function}`);
            console.log(`Parameters: ${result.parameters.length}`);
        });
    })
    .catch(error => {
        console.error('Analysis failed:', error.message);
    });
```

## Performance Considerations

### Memory Usage

- The analyzer loads the entire project into memory
- Large projects with many dependencies may require significant RAM
- Consider analyzing functions in batches for very large projects

### Analysis Time

- Initial project loading is the most time-consuming operation
- Subsequent function analyses on the same project are fast
- Complex dependency graphs may increase analysis time

### Optimization Tips

1. **Reuse analyzer instances** when analyzing multiple functions in the same project
2. **Use quiet mode** (`--quiet`) in scripts to reduce I/O overhead
3. **Filter function lists** to only analyze functions you need
4. **Consider parallel analysis** for multiple projects (but not multiple functions in the same project)

## Troubleshooting

### Common Issues

1. **"Invalid project path" error**
   - Verify the path exists and is readable
   - Ensure the path points to a directory containing Move.toml

2. **"Function not found" error**
   - Check function name spelling and case sensitivity
   - Verify the function exists in the project source code
   - Use verbose mode to see what functions are available

3. **Parse errors**
   - Ensure all Move files have valid syntax
   - Check that all dependencies are properly configured in Move.toml
   - Verify that the Move edition is supported

4. **Type resolution errors**
   - Usually indicate complex generic types that couldn't be resolved
   - The analysis will continue with "unknown" type placeholders
   - Check for missing or incorrect dependency declarations

### Debug Mode

Use verbose flags for detailed debugging information:

```bash
# Basic debugging
move-function-analyzer -p . -f transfer -v

# Maximum verbosity
move-function-analyzer -p . -f transfer -vvv
```

### Performance Issues

If analysis is slow:

1. Check project size and dependency count
2. Verify all dependencies are accessible
3. Consider using a faster storage device
4. Monitor memory usage during analysis

## Version Compatibility

- **Rust Edition**: 2021 or later
- **Move Edition**: 2024.beta, 2024.alpha
- **Sui Framework**: Compatible with current Sui testnet/mainnet versions
- **Dependencies**: See Cargo.toml for specific version requirements