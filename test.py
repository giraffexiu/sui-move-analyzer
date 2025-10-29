import json
import sys
from sui_move_analyzer import MoveFunctionAnalyzer
def main():
    """Test the analyzer with specific project path and function name."""
    project_path = "tests/deepbook"
    function_name = "swap_exact_base_for_quote"
    try:
        analyzer = MoveFunctionAnalyzer()
        raw_data = analyzer.analyze_raw(project_path, function_name)
        print(json.dumps(raw_data, indent=2, ensure_ascii=False))
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
if __name__ == "__main__":
    main()