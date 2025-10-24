#!/usr/bin/env python3
"""
Simple test file to call the Move Function Analyzer library and output raw JSON only.
"""

import json
import sys
from move_function_analyzer import MoveFunctionAnalyzer

def main():
    """Test the analyzer with specific project path and function name."""
    
    # 配置项目路径和函数名称
    project_path = "/Users/giraffe/Downloads/Work/Sui/move-analyer/depository/deepbookv3/packages/deepbook"
    function_name = "place_market_order"
    
    try:
        # 创建分析器实例
        analyzer = MoveFunctionAnalyzer()
        
        # 分析函数并输出原始JSON结果
        raw_data = analyzer.analyze_raw(project_path, function_name)
        
        # 只输出JSON，不要其他信息
        print(json.dumps(raw_data, indent=2, ensure_ascii=False))
        
    except Exception as e:
        # 错误信息输出到stderr，不影响stdout的JSON输出
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()