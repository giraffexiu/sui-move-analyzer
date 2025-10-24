# Move Function Analyzer Examples

This directory contains example Move projects and usage scenarios for the Move Function Analyzer tool.

## Example Projects

### Simple NFT Project

The `simple-nft` directory contains a basic NFT implementation with marketplace functionality. This project demonstrates:

- Basic NFT minting and transfer functions
- Marketplace listing and purchasing
- Complex function call relationships
- Various Move language features

#### Project Structure

```
simple-nft/
├── Move.toml
└── sources/
    ├── nft.move          # Core NFT functionality
    └── marketplace.move  # NFT marketplace
```

## Usage Examples

### Basic Function Analysis

Analyze the `mint` function in the NFT module:

```bash
move-function-analyzer -p examples/simple-nft -f mint --pretty
```

Expected output:
```json
[
  {
    "contract": "simple_nft::nft",
    "function": "mint(name: vector<u8>, description: vector<u8>, ctx: &mut sui::tx_context::TxContext): simple_nft::nft::SimpleNFT",
    "source": "    public fun mint(\n        name: vector<u8>,\n        description: vector<u8>,\n        ctx: &mut TxContext\n    ): SimpleNFT {\n        let sender = tx_context::sender(ctx);\n        let nft = SimpleNFT {\n            id: object::new(ctx),\n            name: string::utf8(name),\n            description: string::utf8(description),\n            creator: sender,\n        };\n\n        // Emit minting event\n        sui::event::emit(NFTMinted {\n            nft_id: object::uid_to_address(&nft.id),\n            name: nft.name,\n            creator: sender,\n        });\n\n        nft\n    }",
    "location": {
      "file": "/path/to/examples/simple-nft/sources/nft.move",
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
        "file": "/path/to/examples/simple-nft/sources/nft.move",
        "function": "sender(ctx: &sui::tx_context::TxContext): address",
        "module": "sui::tx_context"
      },
      {
        "file": "/path/to/examples/simple-nft/sources/nft.move", 
        "function": "new(ctx: &mut sui::tx_context::TxContext): sui::object::UID",
        "module": "sui::object"
      },
      {
        "file": "/path/to/examples/simple-nft/sources/nft.move",
        "function": "utf8(bytes: vector<u8>): std::string::String",
        "module": "std::string"
      }
    ]
  }
]
```

### Analyzing Complex Functions

Analyze the `purchase` function which has complex call relationships:

```bash
move-function-analyzer -p examples/simple-nft -f purchase --pretty
```

This will show:
- Multiple function calls within the implementation
- Complex parameter types including generics
- Event emission calls
- Object manipulation functions

### Finding Multiple Functions

Analyze the `transfer` function which exists in multiple contexts:

```bash
move-function-analyzer -p examples/simple-nft -f transfer --pretty
```

This will return multiple results:
- The `transfer` function in the NFT module
- Any other `transfer` functions found in the project

### Verbose Analysis

Get detailed information about the analysis process:

```bash
move-function-analyzer -p examples/simple-nft -f mint_and_transfer -v
```

This will show:
- Project loading progress
- Function discovery process
- Detailed analysis steps
- Any warnings or issues encountered

## Common Use Cases

### 1. Understanding Function Dependencies

Use the analyzer to understand what functions a particular function calls:

```bash
# Analyze a complex function to see its dependencies
move-function-analyzer -p examples/simple-nft -f buy_nft --pretty
```

The `calls` array in the output will show all functions called by `buy_nft`, helping you understand the dependency graph.

### 2. Code Documentation

Generate documentation for functions:

```bash
# Get complete function signatures and source code
move-function-analyzer -p examples/simple-nft -f create_listing --pretty > create_listing_docs.json
```

### 3. Code Review and Analysis

Analyze functions before code review:

```bash
# Check all entry functions
for func in mint_and_transfer list_nft buy_nft; do
    echo "=== Analyzing $func ==="
    move-function-analyzer -p examples/simple-nft -f "$func" --quiet
done
```

### 4. Integration Testing

Use the analyzer to understand function interfaces for testing:

```bash
# Get parameter information for test setup
move-function-analyzer -p examples/simple-nft -f purchase --quiet | jq '.[0].parameters'
```

## Scripting Examples

### Bash Script: Analyze All Functions

```bash
#!/bin/bash
# analyze-all.sh - Analyze all functions in a project

PROJECT_PATH="$1"
OUTPUT_DIR="analysis_results"

if [ -z "$PROJECT_PATH" ]; then
    echo "Usage: $0 <project_path>"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

# List of functions to analyze (you would typically extract this from the source)
FUNCTIONS=(
    "mint"
    "transfer" 
    "burn"
    "mint_and_transfer"
    "create_listing"
    "purchase"
    "list_nft"
    "buy_nft"
)

echo "Analyzing functions in $PROJECT_PATH..."

for func in "${FUNCTIONS[@]}"; do
    echo "Analyzing function: $func"
    output_file="$OUTPUT_DIR/${func}_analysis.json"
    
    if move-function-analyzer -p "$PROJECT_PATH" -f "$func" --pretty > "$output_file" 2>/dev/null; then
        echo "  ✓ Analysis saved to $output_file"
    else
        echo "  ✗ Failed to analyze $func"
        rm -f "$output_file"
    fi
done

echo "Analysis complete. Results in $OUTPUT_DIR/"
```

### Python Script: Function Complexity Analysis

```python
#!/usr/bin/env python3
"""
Analyze function complexity based on the number of calls and parameters.
"""

import subprocess
import json
import sys
from pathlib import Path

def analyze_function_complexity(project_path, function_name):
    """Analyze a function and return complexity metrics."""
    try:
        result = subprocess.run([
            'move-function-analyzer',
            '--project-path', str(project_path),
            '--function', function_name,
            '--quiet'
        ], capture_output=True, text=True, check=True)
        
        analyses = json.loads(result.stdout)
        
        complexities = []
        for analysis in analyses:
            complexity = {
                'function': analysis['function'],
                'module': analysis['contract'],
                'parameter_count': len(analysis['parameters']),
                'call_count': len(analysis['calls']),
                'line_count': analysis['location']['end_line'] - analysis['location']['start_line'] + 1,
                'complexity_score': len(analysis['parameters']) + len(analysis['calls']) * 2
            }
            complexities.append(complexity)
        
        return complexities
        
    except subprocess.CalledProcessError as e:
        print(f"Analysis failed for {function_name}: {e.stderr}", file=sys.stderr)
        return []
    except json.JSONDecodeError as e:
        print(f"Failed to parse JSON for {function_name}: {e}", file=sys.stderr)
        return []

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 complexity_analysis.py <project_path>")
        sys.exit(1)
    
    project_path = Path(sys.argv[1])
    
    # Functions to analyze
    functions = [
        'mint', 'transfer', 'burn', 'mint_and_transfer',
        'create_listing', 'purchase', 'list_nft', 'buy_nft'
    ]
    
    all_complexities = []
    
    for func_name in functions:
        print(f"Analyzing {func_name}...", file=sys.stderr)
        complexities = analyze_function_complexity(project_path, func_name)
        all_complexities.extend(complexities)
    
    # Sort by complexity score
    all_complexities.sort(key=lambda x: x['complexity_score'], reverse=True)
    
    # Output results
    print("Function Complexity Analysis")
    print("=" * 50)
    print(f"{'Function':<30} {'Params':<8} {'Calls':<8} {'Lines':<8} {'Score':<8}")
    print("-" * 50)
    
    for comp in all_complexities:
        print(f"{comp['function']:<30} {comp['parameter_count']:<8} {comp['call_count']:<8} {comp['line_count']:<8} {comp['complexity_score']:<8}")

if __name__ == '__main__':
    main()
```

### Node.js Script: Call Graph Generator

```javascript
#!/usr/bin/env node
/**
 * Generate a call graph from Move function analysis results.
 */

const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

async function analyzeFunction(projectPath, functionName) {
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
                    resolve([]); // Return empty array if parsing fails
                }
            } else {
                resolve([]); // Return empty array on error
            }
        });
    });
}

async function generateCallGraph(projectPath, functions) {
    const callGraph = {
        nodes: [],
        edges: []
    };
    
    const nodeSet = new Set();
    
    for (const funcName of functions) {
        console.error(`Analyzing ${funcName}...`);
        const results = await analyzeFunction(projectPath, funcName);
        
        for (const result of results) {
            const nodeId = `${result.contract}::${funcName}`;
            
            if (!nodeSet.has(nodeId)) {
                callGraph.nodes.push({
                    id: nodeId,
                    label: funcName,
                    module: result.contract,
                    parameterCount: result.parameters.length,
                    lineCount: result.location.end_line - result.location.start_line + 1
                });
                nodeSet.add(nodeId);
            }
            
            // Add edges for function calls
            for (const call of result.calls) {
                const targetId = `${call.module}::${call.function.split('(')[0]}`;
                callGraph.edges.push({
                    source: nodeId,
                    target: targetId,
                    type: 'calls'
                });
            }
        }
    }
    
    return callGraph;
}

async function main() {
    if (process.argv.length !== 3) {
        console.error('Usage: node call_graph.js <project_path>');
        process.exit(1);
    }
    
    const projectPath = process.argv[2];
    
    const functions = [
        'mint', 'transfer', 'burn', 'mint_and_transfer',
        'create_listing', 'purchase', 'list_nft', 'buy_nft'
    ];
    
    try {
        const callGraph = await generateCallGraph(projectPath, functions);
        
        // Output as JSON
        console.log(JSON.stringify(callGraph, null, 2));
        
        // Also save to file
        await fs.writeFile('call_graph.json', JSON.stringify(callGraph, null, 2));
        console.error('Call graph saved to call_graph.json');
        
    } catch (error) {
        console.error('Error generating call graph:', error.message);
        process.exit(1);
    }
}

main();
```

## Testing the Examples

### Prerequisites

1. Build the Move Function Analyzer:
   ```bash
   cd src/beta-2024
   cargo build --release --bin move-function-analyzer
   ```

2. Add the binary to your PATH or use the full path.

### Running the Examples

1. **Basic analysis**:
   ```bash
   move-function-analyzer -p examples/simple-nft -f mint --pretty
   ```

2. **Analyze all functions**:
   ```bash
   ./examples/analyze-all.sh examples/simple-nft
   ```

3. **Complexity analysis**:
   ```bash
   python3 examples/complexity_analysis.py examples/simple-nft
   ```

4. **Call graph generation**:
   ```bash
   node examples/call_graph.js examples/simple-nft
   ```

## Expected Results

When running the examples, you should see:

- **JSON output** with complete function information
- **Parameter details** including complex Move types
- **Call relationships** showing function dependencies
- **Location information** with accurate line numbers
- **Source code** with original formatting preserved

## Troubleshooting Examples

If you encounter issues:

1. **Build errors**: Ensure you have the latest Rust toolchain and Move dependencies
2. **Path issues**: Use absolute paths or ensure the binary is in your PATH
3. **Project errors**: Verify the example project has valid Move.toml and source files
4. **Permission errors**: Ensure you have read access to the project directory

## Creating Your Own Examples

To create additional examples:

1. Create a new directory under `examples/`
2. Add a valid `Move.toml` file
3. Create Move source files in the `sources/` directory
4. Test with the analyzer tool
5. Document the expected behavior

The analyzer works with any valid Sui Move project, so you can use your own projects as examples as well.