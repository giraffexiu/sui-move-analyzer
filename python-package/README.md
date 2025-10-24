# Move Function Analyzer

A Python library for analyzing Move functions in Sui Move projects. This library provides a simple interface to extract detailed information about Move functions including source code, parameters, location information, and function call relationships.

## Installation

```bash
pip install move-function-analyzer
```

## Quick Start

```python
from move_function_analyzer import MoveFunctionAnalyzer

# Create an analyzer instance
analyzer = MoveFunctionAnalyzer()

# Analyze a function
results = analyzer.analyze("/path/to/move/project", "transfer")

# Print results
for result in results:
    print(f"Contract: {result.contract}")
    print(f"Function: {result.function}")
    print(f"Parameters: {[p.name for p in result.parameters]}")
    print(f"Calls: {len(result.calls)} function calls")
    print("---")
```

## Features

- **Simple API**: Easy-to-use Python interface
- **Detailed Analysis**: Extract function signatures, source code, parameters, and call graphs
- **Multiple Results**: Handle functions with the same name in different modules
- **Error Handling**: Comprehensive exception handling with specific error types
- **Type Safety**: Full type hints for better IDE support

## API Reference

### MoveFunctionAnalyzer

The main analyzer class.

```python
analyzer = MoveFunctionAnalyzer()
```

#### Methods

##### `analyze(project_path, function_name)`

Analyze a Move function and return structured results.

**Parameters:**
- `project_path` (str | Path): Path to the Move project directory (containing Move.toml)
- `function_name` (str): Name of the function to analyze

**Returns:**
- `List[AnalysisResult]`: List of analysis results (multiple if function exists in different modules)

**Raises:**
- `ProjectNotFoundError`: If the project path doesn't exist or is invalid
- `FunctionNotFoundError`: If the function is not found in the project
- `AnalysisFailedError`: If the analysis process fails

##### `analyze_raw(project_path, function_name)`

Analyze a Move function and return raw JSON data.

**Parameters:**
- Same as `analyze()`

**Returns:**
- `Dict[str, Any]`: Raw JSON data as dictionary

### Data Classes

#### `AnalysisResult`

Contains complete analysis information for a function.

**Attributes:**
- `contract` (str): Module name containing the function
- `function` (str): Function signature with parameters and return type
- `source` (str): Complete source code of the function
- `location` (LocationInfo): File location information
- `parameters` (List[Parameter]): Function parameters
- `calls` (List[FunctionCall]): Functions called by this function

#### `LocationInfo`

File location information.

**Attributes:**
- `file` (str): Path to the source file
- `start_line` (int): Starting line number
- `end_line` (int): Ending line number

#### `Parameter`

Function parameter information.

**Attributes:**
- `name` (str): Parameter name
- `type` (str): Parameter type

#### `FunctionCall`

Information about function calls.

**Attributes:**
- `file` (str): File containing the called function
- `function` (str): Function signature
- `module` (str): Module name

### Convenience Functions

#### `analyze_function(project_path, function_name)`

Quick analysis without creating an analyzer instance.

```python
from move_function_analyzer import analyze_function

results = analyze_function("/path/to/project", "mint")
```

## Examples

### Basic Usage

```python
from move_function_analyzer import MoveFunctionAnalyzer

analyzer = MoveFunctionAnalyzer()

try:
    results = analyzer.analyze("./my-move-project", "transfer")
    
    for result in results:
        print(f"Found in module: {result.contract}")
        print(f"Function signature: {result.function}")
        print(f"Source code length: {len(result.source)} characters")
        print(f"Located at: {result.location.file}:{result.location.start_line}")
        
        print("Parameters:")
        for param in result.parameters:
            print(f"  - {param.name}: {param.type}")
        
        print("Function calls:")
        for call in result.calls:
            print(f"  - {call.function} in {call.module}")
        
        print("-" * 50)

except Exception as e:
    print(f"Analysis failed: {e}")
```

### Error Handling

```python
from move_function_analyzer import (
    MoveFunctionAnalyzer, 
    ProjectNotFoundError, 
    FunctionNotFoundError,
    AnalysisFailedError
)

analyzer = MoveFunctionAnalyzer()

try:
    results = analyzer.analyze("/path/to/project", "nonexistent_function")
except ProjectNotFoundError as e:
    print(f"Project not found: {e.project_path}")
except FunctionNotFoundError as e:
    print(f"Function '{e.function_name}' not found in {e.project_path}")
except AnalysisFailedError as e:
    print(f"Analysis failed: {e}")
```

### Working with Raw JSON

```python
from move_function_analyzer import MoveFunctionAnalyzer
import json

analyzer = MoveFunctionAnalyzer()
raw_data = analyzer.analyze_raw("./project", "mint")

# Pretty print the JSON
print(json.dumps(raw_data, indent=2))
```

## Requirements

- Python 3.7+
- The analyzer binary is automatically included with the package

## License

Apache License 2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.