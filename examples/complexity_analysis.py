#!/usr/bin/env python3
"""
Move Function Complexity Analysis

This script analyzes Move functions and provides complexity metrics based on:
- Number of parameters
- Number of function calls
- Lines of code
- Calculated complexity score

Usage: python3 complexity_analysis.py <project_path>
"""

import subprocess
import json
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional

class FunctionComplexityAnalyzer:
    """Analyzer for Move function complexity metrics."""
    
    def __init__(self, analyzer_binary: str = 'move-function-analyzer'):
        """Initialize the analyzer with the binary path."""
        self.analyzer_binary = analyzer_binary
    
    def analyze_function(self, project_path: Path, function_name: str) -> List[Dict[str, Any]]:
        """
        Analyze a single function and return complexity metrics.
        
        Args:
            project_path: Path to the Move project
            function_name: Name of the function to analyze
            
        Returns:
            List of complexity analysis results
        """
        try:
            result = subprocess.run([
                self.analyzer_binary,
                '--project-path', str(project_path),
                '--function', function_name,
                '--quiet'
            ], capture_output=True, text=True, check=True, timeout=30)
            
            analyses = json.loads(result.stdout)
            
            complexities = []
            for analysis in analyses:
                complexity = self._calculate_complexity_metrics(analysis)
                complexities.append(complexity)
            
            return complexities
            
        except subprocess.CalledProcessError as e:
            print(f"Analysis failed for {function_name}: {e.stderr}", file=sys.stderr)
            return []
        except subprocess.TimeoutExpired:
            print(f"Analysis timed out for {function_name}", file=sys.stderr)
            return []
        except json.JSONDecodeError as e:
            print(f"Failed to parse JSON for {function_name}: {e}", file=sys.stderr)
            return []
        except Exception as e:
            print(f"Unexpected error analyzing {function_name}: {e}", file=sys.stderr)
            return []
    
    def _calculate_complexity_metrics(self, analysis: Dict[str, Any]) -> Dict[str, Any]:
        """Calculate complexity metrics from function analysis."""
        parameter_count = len(analysis['parameters'])
        call_count = len(analysis['calls'])
        line_count = analysis['location']['end_line'] - analysis['location']['start_line'] + 1
        
        # Calculate complexity score using weighted factors
        complexity_score = (
            parameter_count * 1 +      # Each parameter adds 1 point
            call_count * 2 +           # Each function call adds 2 points
            max(0, line_count - 5) * 0.1  # Lines over 5 add 0.1 points each
        )
        
        return {
            'function': analysis['function'],
            'module': analysis['contract'],
            'parameter_count': parameter_count,
            'call_count': call_count,
            'line_count': line_count,
            'complexity_score': round(complexity_score, 1),
            'file_path': str(analysis['location']['file']),
            'start_line': analysis['location']['start_line'],
            'end_line': analysis['location']['end_line'],
            'parameters': [p['name'] for p in analysis['parameters']],
            'called_functions': [call['function'].split('(')[0] for call in analysis['calls']]
        }
    
    def analyze_project(self, project_path: Path, functions: List[str]) -> List[Dict[str, Any]]:
        """
        Analyze multiple functions in a project.
        
        Args:
            project_path: Path to the Move project
            functions: List of function names to analyze
            
        Returns:
            List of all complexity analysis results
        """
        all_complexities = []
        
        for func_name in functions:
            print(f"Analyzing {func_name}...", file=sys.stderr)
            complexities = self.analyze_function(project_path, func_name)
            all_complexities.extend(complexities)
        
        return all_complexities
    
    def generate_report(self, complexities: List[Dict[str, Any]], output_format: str = 'table') -> str:
        """
        Generate a complexity report in the specified format.
        
        Args:
            complexities: List of complexity analysis results
            output_format: Format for the report ('table', 'json', 'csv')
            
        Returns:
            Formatted report string
        """
        if not complexities:
            return "No complexity data available."
        
        # Sort by complexity score (descending)
        sorted_complexities = sorted(complexities, key=lambda x: x['complexity_score'], reverse=True)
        
        if output_format == 'json':
            return json.dumps(sorted_complexities, indent=2)
        elif output_format == 'csv':
            return self._generate_csv_report(sorted_complexities)
        else:  # table format
            return self._generate_table_report(sorted_complexities)
    
    def _generate_table_report(self, complexities: List[Dict[str, Any]]) -> str:
        """Generate a table format report."""
        lines = []
        lines.append("Move Function Complexity Analysis")
        lines.append("=" * 80)
        lines.append("")
        
        # Summary statistics
        total_functions = len(complexities)
        avg_complexity = sum(c['complexity_score'] for c in complexities) / total_functions
        max_complexity = max(c['complexity_score'] for c in complexities)
        min_complexity = min(c['complexity_score'] for c in complexities)
        
        lines.append(f"Summary:")
        lines.append(f"  Total functions analyzed: {total_functions}")
        lines.append(f"  Average complexity score: {avg_complexity:.1f}")
        lines.append(f"  Maximum complexity score: {max_complexity}")
        lines.append(f"  Minimum complexity score: {min_complexity}")
        lines.append("")
        
        # Detailed table
        header = f"{'Function':<40} {'Module':<25} {'Params':<7} {'Calls':<7} {'Lines':<7} {'Score':<7}"
        lines.append(header)
        lines.append("-" * len(header))
        
        for comp in complexities:
            func_name = comp['function'].split('(')[0]  # Get just the function name
            if len(func_name) > 39:
                func_name = func_name[:36] + "..."
            
            module_name = comp['module']
            if len(module_name) > 24:
                module_name = module_name[:21] + "..."
            
            line = f"{func_name:<40} {module_name:<25} {comp['parameter_count']:<7} {comp['call_count']:<7} {comp['line_count']:<7} {comp['complexity_score']:<7}"
            lines.append(line)
        
        lines.append("")
        
        # Complexity categories
        high_complexity = [c for c in complexities if c['complexity_score'] >= 10]
        medium_complexity = [c for c in complexities if 5 <= c['complexity_score'] < 10]
        low_complexity = [c for c in complexities if c['complexity_score'] < 5]
        
        lines.append("Complexity Categories:")
        lines.append(f"  High complexity (≥10): {len(high_complexity)} functions")
        lines.append(f"  Medium complexity (5-9): {len(medium_complexity)} functions")
        lines.append(f"  Low complexity (<5): {len(low_complexity)} functions")
        
        if high_complexity:
            lines.append("")
            lines.append("High Complexity Functions (may need refactoring):")
            for comp in high_complexity[:5]:  # Show top 5
                func_name = comp['function'].split('(')[0]
                lines.append(f"  - {func_name} (score: {comp['complexity_score']})")
                lines.append(f"    Module: {comp['module']}")
                lines.append(f"    Parameters: {comp['parameter_count']}, Calls: {comp['call_count']}, Lines: {comp['line_count']}")
        
        return "\n".join(lines)
    
    def _generate_csv_report(self, complexities: List[Dict[str, Any]]) -> str:
        """Generate a CSV format report."""
        lines = []
        lines.append("Function,Module,Parameters,Calls,Lines,Complexity_Score,File_Path,Start_Line,End_Line")
        
        for comp in complexities:
            func_name = comp['function'].split('(')[0].replace(',', ';')  # Escape commas
            module_name = comp['module'].replace(',', ';')
            file_path = comp['file_path'].replace(',', ';')
            
            line = f"{func_name},{module_name},{comp['parameter_count']},{comp['call_count']},{comp['line_count']},{comp['complexity_score']},{file_path},{comp['start_line']},{comp['end_line']}"
            lines.append(line)
        
        return "\n".join(lines)

def main():
    """Main entry point for the complexity analysis tool."""
    parser = argparse.ArgumentParser(
        description="Analyze Move function complexity",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 complexity_analysis.py examples/simple-nft
  python3 complexity_analysis.py examples/simple-nft --format json
  python3 complexity_analysis.py examples/simple-nft --functions mint transfer burn
  python3 complexity_analysis.py examples/simple-nft --output complexity_report.json
        """
    )
    
    parser.add_argument('project_path', type=Path, help='Path to the Move project')
    parser.add_argument('--functions', nargs='+', 
                       default=['mint', 'transfer', 'burn', 'mint_and_transfer',
                               'create_listing', 'purchase', 'list_nft', 'buy_nft',
                               'name', 'description', 'creator', 'price', 'seller',
                               'cancel_listing', 'update_description'],
                       help='List of function names to analyze')
    parser.add_argument('--format', choices=['table', 'json', 'csv'], default='table',
                       help='Output format for the report')
    parser.add_argument('--output', type=Path, help='Output file path (default: stdout)')
    parser.add_argument('--analyzer-binary', default='move-function-analyzer',
                       help='Path to the move-function-analyzer binary')
    
    args = parser.parse_args()
    
    # Validate project path
    if not args.project_path.exists():
        print(f"Error: Project path '{args.project_path}' does not exist", file=sys.stderr)
        sys.exit(1)
    
    if not (args.project_path / 'Move.toml').exists():
        print(f"Error: Move.toml not found in '{args.project_path}'", file=sys.stderr)
        sys.exit(1)
    
    # Check if analyzer binary is available
    try:
        subprocess.run([args.analyzer_binary, '--version'], 
                      capture_output=True, check=True, timeout=5)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, FileNotFoundError):
        print(f"Error: '{args.analyzer_binary}' command not found or not working", file=sys.stderr)
        print("Please ensure the binary is built and in your PATH:", file=sys.stderr)
        print("  cd src/beta-2024", file=sys.stderr)
        print("  cargo build --release --bin move-function-analyzer", file=sys.stderr)
        print("  export PATH=$PATH:$(pwd)/target/release", file=sys.stderr)
        sys.exit(1)
    
    # Perform analysis
    analyzer = FunctionComplexityAnalyzer(args.analyzer_binary)
    
    print(f"Analyzing {len(args.functions)} functions in {args.project_path}...", file=sys.stderr)
    complexities = analyzer.analyze_project(args.project_path, args.functions)
    
    if not complexities:
        print("No functions were successfully analyzed.", file=sys.stderr)
        sys.exit(1)
    
    # Generate report
    report = analyzer.generate_report(complexities, args.format)
    
    # Output report
    if args.output:
        try:
            args.output.write_text(report)
            print(f"Report saved to {args.output}", file=sys.stderr)
        except Exception as e:
            print(f"Error writing to {args.output}: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print(report)

if __name__ == '__main__':
    main()