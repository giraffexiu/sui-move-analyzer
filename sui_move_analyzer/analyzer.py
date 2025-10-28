"""
Main analyzer module for the Move Function Analyzer library.
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import List, Dict, Any, Optional, Union
from dataclasses import dataclass

from .exceptions import (
    AnalyzerError, 
    ProjectNotFoundError, 
    FunctionNotFoundError, 
    BinaryNotFoundError,
    AnalysisFailedError
)


@dataclass
class LocationInfo:
    """Information about the location of a function in source code."""
    file: str
    start_line: int
    end_line: int


@dataclass
class Parameter:
    """Information about a function parameter."""
    name: str
    type: str


@dataclass
class FunctionCall:
    """Information about a function call made within the analyzed function."""
    file: str
    function: str
    module: str


@dataclass
class AnalysisResult:
    """Complete analysis result for a Move function."""
    contract: str
    function: str
    source: str
    location: LocationInfo
    parameters: List[Parameter]
    calls: List[FunctionCall]
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'AnalysisResult':
        """Create an AnalysisResult from a dictionary (JSON data)."""
        location = LocationInfo(
            file=data['location']['file'],
            start_line=data['location']['start_line'],
            end_line=data['location']['end_line']
        )
        
        parameters = [
            Parameter(name=param['name'], type=param['type'])
            for param in data['parameter']
        ]
        
        calls = [
            FunctionCall(
                file=call['file'],
                function=call['function'],
                module=call['module']
            )
            for call in data['calls']
        ]
        
        return cls(
            contract=data['contract'],
            function=data['function'],
            source=data['source'],
            location=location,
            parameters=parameters,
            calls=calls
        )


class MoveFunctionAnalyzer:
    """
    Main analyzer class for analyzing Move functions in Sui Move projects.
    
    This class provides a Python interface to the underlying Rust-based analyzer,
    allowing you to analyze Move functions and extract detailed information.
    
    Example:
        >>> analyzer = MoveFunctionAnalyzer()
        >>> results = analyzer.analyze("/path/to/project", "transfer")
        >>> for result in results:
        ...     print(f"Found function: {result.function}")
        ...     print(f"Parameters: {[p.name for p in result.parameters]}")
    """
    
    def __init__(self, binary_path: Optional[str] = None):
        """
        Initialize the analyzer.
        
        Args:
            binary_path: Optional path to the analyzer binary. If not provided,
                        the bundled binary will be used.
        
        Raises:
            BinaryNotFoundError: If the analyzer binary cannot be found.
        """
        if binary_path is None:
            # Use the bundled binary
            package_dir = Path(__file__).parent
            binary_name = "move-function-analyzer"
            if sys.platform == "win32":
                binary_name += ".exe"
            
            self.binary_path = package_dir / "bin" / binary_name
        else:
            self.binary_path = Path(binary_path)
        
        if not self.binary_path.exists():
            raise BinaryNotFoundError(str(self.binary_path))
        
        # Make sure the binary is executable on Unix systems
        if sys.platform != "win32":
            os.chmod(self.binary_path, 0o755)
    
    def analyze(
        self, 
        project_path: Union[str, Path], 
        function_name: str
    ) -> List[AnalysisResult]:
        """
        Analyze a Move function in the specified project.
        
        Args:
            project_path: Path to the Move project directory (containing Move.toml)
            function_name: Name of the function to analyze
        
        Returns:
            List of AnalysisResult objects containing detailed function information.
            Multiple results may be returned if the function name exists in multiple modules.
        
        Raises:
            ProjectNotFoundError: If the project path does not exist
            FunctionNotFoundError: If the function is not found in the project
            AnalysisFailedError: If the analysis process fails
        """
        project_path = Path(project_path).resolve()
        
        # Validate project path
        if not project_path.exists():
            raise ProjectNotFoundError(str(project_path))
        
        if not (project_path / "Move.toml").exists():
            raise ProjectNotFoundError(f"No Move.toml found in {project_path}")
        
        # Run the analyzer
        try:
            result = subprocess.run(
                [str(self.binary_path), str(project_path), function_name],
                capture_output=True,
                text=True,
                check=False  # We'll handle the return code ourselves
            )
        except FileNotFoundError:
            raise BinaryNotFoundError(str(self.binary_path))
        except Exception as e:
            raise AnalysisFailedError(f"Failed to execute analyzer: {e}")
        
        # Handle different return codes
        if result.returncode != 0:
            if result.returncode == 1:
                # Check if it's a "function not found" case by looking at the output
                if result.stdout.strip() == "[]":
                    raise FunctionNotFoundError(function_name, str(project_path))
                else:
                    # Some other error occurred
                    error_msg = result.stderr.strip() if result.stderr.strip() else "Analysis failed"
                    raise AnalysisFailedError(error_msg, result.returncode)
            else:
                error_msg = result.stderr.strip() if result.stderr.strip() else "Analysis failed"
                raise AnalysisFailedError(error_msg, result.returncode)
        
        # Parse the JSON output
        try:
            json_data = json.loads(result.stdout)
        except json.JSONDecodeError as e:
            raise AnalysisFailedError(f"Failed to parse analyzer output as JSON: {e}")
        
        # Check if no functions were found
        if not json_data:
            raise FunctionNotFoundError(function_name, str(project_path))
        
        # Convert to AnalysisResult objects
        try:
            results = [AnalysisResult.from_dict(item) for item in json_data]
        except (KeyError, TypeError) as e:
            raise AnalysisFailedError(f"Invalid analyzer output format: {e}")
        
        return results
    
    def analyze_raw(
        self, 
        project_path: Union[str, Path], 
        function_name: str
    ) -> Dict[str, Any]:
        """
        Analyze a Move function and return raw JSON data.
        
        This method returns the raw JSON output from the analyzer without
        converting it to Python objects. Useful if you need the original format.
        
        Args:
            project_path: Path to the Move project directory (containing Move.toml)
            function_name: Name of the function to analyze
        
        Returns:
            Raw JSON data as a dictionary
        
        Raises:
            Same exceptions as analyze()
        """
        results = self.analyze(project_path, function_name)
        
        # Convert back to dict format for raw output
        raw_results = []
        for result in results:
            raw_result = {
                "contract": result.contract,
                "function": result.function,
                "source": result.source,
                "location": {
                    "file": result.location.file,
                    "start_line": result.location.start_line,
                    "end_line": result.location.end_line
                },
                "parameter": [
                    {"name": param.name, "type": param.type}
                    for param in result.parameters
                ],
                "calls": [
                    {
                        "file": call.file,
                        "function": call.function,
                        "module": call.module
                    }
                    for call in result.calls
                ]
            }
            raw_results.append(raw_result)
        
        return raw_results


# Convenience function for quick analysis
def analyze_function(
    project_path: Union[str, Path], 
    function_name: str
) -> List[AnalysisResult]:
    """
    Convenience function to quickly analyze a Move function.
    
    This is equivalent to creating a MoveFunctionAnalyzer instance and calling analyze().
    
    Args:
        project_path: Path to the Move project directory (containing Move.toml)
        function_name: Name of the function to analyze
    
    Returns:
        List of AnalysisResult objects containing detailed function information
    
    Raises:
        Same exceptions as MoveFunctionAnalyzer.analyze()
    """
    analyzer = MoveFunctionAnalyzer()
    return analyzer.analyze(project_path, function_name)