# Move Function Analyzer

Move Function Analyzer is a powerful tool for analyzing Move functions in Sui Move projects. It extracts detailed information about functions including source code, parameters, location information, and function call relationships.

## Features

- **Function Discovery**: Find functions by name across all modules in a project
- **Source Code Extraction**: Get complete function source code with original formatting
- **Parameter Analysis**: Extract parameter names and types with full type information
- **Call Graph Analysis**: Identify all functions called by the analyzed function
- **Location Information**: Get precise file paths and line numbers
- **JSON Output**: Structured output for easy integration with other tools
- **Multiple Results**: Handle multiple functions with the same name

## Installation

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd sui-move-analyzer

# Build the function analyzer
cd src/beta-2024
cargo build --release --bin move-function-analyzer

# The binary will be available at target/release/move-function-analyzer
```

### Using the Binary

After building, you can use the tool directly:

```bash
# Add to PATH for convenience
export PATH=$PATH:/path/to/sui-move-analyzer/src/beta-2024/target/release

# Or use the full path
./target/release/move-function-analyzer --help
```

## Usage

### Basic Usage

```bash
move-function-analyzer --project-path <PROJECT_PATH> --function <FUNCTION_NAME>
```

### Command Line Options

- `-p, --project-path <PATH>`: Path to the Sui Move project directory (required)
- `-f, --function <NAME>`: Name of the function to analyze (required)
- `--pretty`: Format JSON output with indentation for better readability
- `-v, --verbose`: Enable verbose logging (can be used multiple times for more verbosity)
- `-q, --quiet`: Suppress non-error output
- `-h, --help`: Show help information
- `-V, --version`: Show version information

### Examples

#### Basic Function Analysis

```bash
# Analyze a function named 'transfer' in the current directory
move-function-analyzer -p . -f transfer

# Analyze with pretty-printed JSON output
move-function-analyzer -p ./my-project -f mint --pretty

# Analyze with verbose logging
move-function-analyzer -p /path/to/project -f "public_transfer" --verbose
```

#### Advanced Usage

```bash
# Quiet mode - only show results
move-function-analyzer -p . -f transfer --quiet

# Maximum verbosity for debugging
move-function-analyzer -p . -f transfer -vvv

# Analyze a function in a specific project
move-function-analyzer --project-path /home/user/sui-projects/nft-marketplace --function create_listing --pretty
```

## Output Format

The tool outputs JSON containing an array of function analysis results. Each result includes:

```json
[
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
]
```

### Field Descriptions

- **contract**: The name of the module containing the function
- **function**: Complete function signature including parameters and return type
- **source**: Full source code of the function with original formatting
- **location**: File location information
  - **file**: Absolute path to the source file
  - **start_line**: Starting line number (1-indexed)
  - **end_line**: Ending line number (1-indexed)
- **parameters**: Array of function parameters
  - **name**: Parameter name
  - **type**: Parameter type as a string
- **calls**: Array of functions called by this function
  - **file**: Path to file containing the called function
  - **function**: Signature of the called function
  - **module**: Module name containing the called function

## Project Requirements

The tool requires a valid Sui Move project with:

1. **Move.toml file**: Must be present in the project root
2. **Valid project structure**: Standard Move project layout
3. **Readable source files**: All .move files must be accessible
4. **Valid Move syntax**: Source files must compile successfully

### Supported Project Structures

```
my-project/
├── Move.toml
├── sources/
│   ├── module1.move
│   └── module2.move
├── tests/           # Optional
│   └── test1.move
└── scripts/         # Optional
    └── script1.move
```

## API Documentation

### Library Usage

The function analyzer can also be used as a library in Rust projects:

```rust
use beta_2024::function_analyzer::{FunctionAnalyzer, AnalyzerError};
use std::path::PathBuf;

fn analyze_function() -> Result<(), AnalyzerError> {
    // Create analyzer for a project
    let analyzer = FunctionAnalyzer::new(PathBuf::from("./my-project"))?;
    
    // Analyze a function
    let results = analyzer.analyze_function("transfer")?;
    
    // Process results
    for result in results {
        println!("Found function: {}", result.function);
        println!("Location: {}:{}-{}", 
                 result.location.file.display(),
                 result.location.start_line,
                 result.location.end_line);
    }
    
    Ok(())
}
```

### Error Handling

The tool provides detailed error messages for common issues:

- **Invalid project path**: Project directory doesn't exist or isn't accessible
- **Missing Move.toml**: Project doesn't contain a valid Move.toml file
- **Parse errors**: Move source files contain syntax errors
- **Function not found**: No functions with the specified name exist
- **Type resolution errors**: Issues resolving complex types

## Troubleshooting

### Common Issues

1. **"Invalid project path" error**
   - Ensure the path exists and is accessible
   - Check that the path points to a directory, not a file
   - Verify you have read permissions for the directory

2. **"Move.toml file not found" error**
   - Ensure Move.toml exists in the project root
   - Check that Move.toml is a valid file (not a directory)
   - Verify the file is readable

3. **"Function not found" error**
   - Check the function name spelling
   - Ensure the function exists in the project
   - Try using verbose mode to see what functions are available

4. **Parse errors**
   - Ensure all Move files have valid syntax
   - Check for missing dependencies in Move.toml
   - Use verbose mode to see detailed error information

### Debug Mode

Use verbose flags to get detailed information about the analysis process:

```bash
# Basic verbose output
move-function-analyzer -p . -f transfer -v

# Maximum verbosity for debugging
move-function-analyzer -p . -f transfer -vvv
```

### Performance Considerations

- Large projects may take longer to analyze
- The tool loads the entire project into memory
- Complex dependency graphs may increase analysis time
- Use quiet mode (`--quiet`) for faster execution in scripts

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# analyze-all-functions.sh

PROJECT_PATH="$1"
FUNCTIONS=("transfer" "mint" "burn" "create")

for func in "${FUNCTIONS[@]}"; do
    echo "Analyzing function: $func"
    move-function-analyzer -p "$PROJECT_PATH" -f "$func" --pretty > "${func}_analysis.json"
done
```

### Python Integration

```python
import subprocess
import json
import sys

def analyze_function(project_path, function_name):
    """Analyze a Move function and return parsed results."""
    try:
        result = subprocess.run([
            'move-function-analyzer',
            '--project-path', project_path,
            '--function', function_name,
            '--quiet'
        ], capture_output=True, text=True, check=True)
        
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Analysis failed: {e.stderr}", file=sys.stderr)
        return None
    except json.JSONDecodeError as e:
        print(f"Failed to parse JSON output: {e}", file=sys.stderr)
        return None

# Usage
results = analyze_function('./my-project', 'transfer')
if results:
    for result in results:
        print(f"Function: {result['function']}")
        print(f"Module: {result['contract']}")
        print(f"Parameters: {len(result['parameters'])}")
```

### Node.js Integration

```javascript
const { spawn } = require('child_process');

function analyzeFunction(projectPath, functionName) {
    return new Promise((resolve, reject) => {
        const analyzer = spawn('move-function-analyzer', [
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
                    const results = JSON.parse(output);
                    resolve(results);
                } catch (e) {
                    reject(new Error(`Failed to parse JSON: ${e.message}`));
                }
            } else {
                reject(new Error(`Analysis failed: ${error}`));
            }
        });
    });
}

// Usage
analyzeFunction('./my-project', 'transfer')
    .then(results => {
        results.forEach(result => {
            console.log(`Function: ${result.function}`);
            console.log(`Module: ${result.contract}`);
            console.log(`Parameters: ${result.parameters.length}`);
        });
    })
    .catch(error => {
        console.error('Error:', error.message);
    });
```

## Contributing

Contributions are welcome! Please see the main project documentation for contribution guidelines.

## License

This project is licensed under the Apache License 2.0. See the LICENSE file for details.