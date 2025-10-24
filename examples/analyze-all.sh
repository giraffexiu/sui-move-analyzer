#!/bin/bash
# analyze-all.sh - Analyze all functions in a Move project
# 
# This script demonstrates how to use the Move Function Analyzer
# to analyze multiple functions in a project and save results.

PROJECT_PATH="$1"
OUTPUT_DIR="analysis_results"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Check if project path is provided
if [ -z "$PROJECT_PATH" ]; then
    print_status $RED "Usage: $0 <project_path>"
    echo "Example: $0 examples/simple-nft"
    exit 1
fi

# Check if project path exists
if [ ! -d "$PROJECT_PATH" ]; then
    print_status $RED "Error: Project path '$PROJECT_PATH' does not exist"
    exit 1
fi

# Check if Move.toml exists
if [ ! -f "$PROJECT_PATH/Move.toml" ]; then
    print_status $RED "Error: Move.toml not found in '$PROJECT_PATH'"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# List of common Move functions to analyze
# You can customize this list based on your project
FUNCTIONS=(
    "mint"
    "transfer" 
    "burn"
    "mint_and_transfer"
    "create_listing"
    "purchase"
    "list_nft"
    "buy_nft"
    "name"
    "description"
    "creator"
    "price"
    "seller"
    "cancel_listing"
    "update_description"
)

print_status $BLUE "Move Function Analyzer - Batch Analysis"
print_status $BLUE "======================================"
echo "Project: $PROJECT_PATH"
echo "Output: $OUTPUT_DIR/"
echo ""

# Check if move-function-analyzer is available
if ! command -v move-function-analyzer &> /dev/null; then
    print_status $RED "Error: move-function-analyzer command not found"
    echo "Please ensure the binary is built and in your PATH:"
    echo "  cd src/beta-2024"
    echo "  cargo build --release --bin move-function-analyzer"
    echo "  export PATH=\$PATH:\$(pwd)/target/release"
    exit 1
fi

# Statistics
total_functions=${#FUNCTIONS[@]}
successful_analyses=0
failed_analyses=0

print_status $YELLOW "Analyzing $total_functions functions..."
echo ""

# Analyze each function
for func in "${FUNCTIONS[@]}"; do
    echo -n "Analyzing '$func'... "
    output_file="$OUTPUT_DIR/${func}_analysis.json"
    
    # Run the analyzer with timeout to prevent hanging
    if timeout 30s move-function-analyzer -p "$PROJECT_PATH" -f "$func" --pretty > "$output_file" 2>/dev/null; then
        # Check if the output file contains valid results
        if [ -s "$output_file" ] && jq empty "$output_file" 2>/dev/null; then
            result_count=$(jq length "$output_file" 2>/dev/null || echo "0")
            if [ "$result_count" -gt 0 ]; then
                print_status $GREEN "✓ Found $result_count result(s)"
                ((successful_analyses++))
            else
                print_status $YELLOW "✓ No functions found"
                rm -f "$output_file"
                ((failed_analyses++))
            fi
        else
            print_status $RED "✗ Invalid output"
            rm -f "$output_file"
            ((failed_analyses++))
        fi
    else
        print_status $RED "✗ Analysis failed or timed out"
        rm -f "$output_file"
        ((failed_analyses++))
    fi
done

echo ""
print_status $BLUE "Analysis Summary"
print_status $BLUE "==============="
echo "Total functions analyzed: $total_functions"
print_status $GREEN "Successful analyses: $successful_analyses"
print_status $RED "Failed analyses: $failed_analyses"

if [ $successful_analyses -gt 0 ]; then
    echo ""
    print_status $YELLOW "Generated files:"
    ls -la "$OUTPUT_DIR"/*.json 2>/dev/null | while read -r line; do
        echo "  $line"
    done
    
    echo ""
    print_status $BLUE "Example usage:"
    echo "  # View a specific analysis result"
    echo "  cat $OUTPUT_DIR/mint_analysis.json | jq ."
    echo ""
    echo "  # Extract function signatures"
    echo "  jq -r '.[].function' $OUTPUT_DIR/*.json"
    echo ""
    echo "  # Count parameters for each function"
    echo "  jq -r '.[] | \"\\(.function): \\(.parameters | length) parameters\"' $OUTPUT_DIR/*.json"
fi

# Generate summary report
summary_file="$OUTPUT_DIR/analysis_summary.json"
echo "{" > "$summary_file"
echo "  \"project_path\": \"$PROJECT_PATH\"," >> "$summary_file"
echo "  \"analysis_date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$summary_file"
echo "  \"total_functions_analyzed\": $total_functions," >> "$summary_file"
echo "  \"successful_analyses\": $successful_analyses," >> "$summary_file"
echo "  \"failed_analyses\": $failed_analyses," >> "$summary_file"
echo "  \"results\": [" >> "$summary_file"

first=true
for json_file in "$OUTPUT_DIR"/*.json; do
    if [ "$json_file" != "$summary_file" ] && [ -f "$json_file" ]; then
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$summary_file"
        fi
        echo -n "    {\"file\": \"$(basename "$json_file")\", \"function_count\": " >> "$summary_file"
        jq length "$json_file" >> "$summary_file" | tr -d '\n'
        echo -n "}" >> "$summary_file"
    fi
done

echo "" >> "$summary_file"
echo "  ]" >> "$summary_file"
echo "}" >> "$summary_file"

print_status $GREEN "Summary report saved to: $summary_file"

if [ $successful_analyses -eq 0 ]; then
    print_status $RED "No functions were successfully analyzed. Check your project structure and function names."
    exit 1
fi

print_status $GREEN "Batch analysis completed successfully!"