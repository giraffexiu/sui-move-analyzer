"""
Exception classes for the Move Function Analyzer library.
"""


class AnalyzerError(Exception):
    """Base exception class for all analyzer errors."""
    pass


class ProjectNotFoundError(AnalyzerError):
    """Raised when the specified Move project path does not exist or is invalid."""
    
    def __init__(self, project_path: str):
        self.project_path = project_path
        super().__init__(f"Move project not found at path: {project_path}")


class FunctionNotFoundError(AnalyzerError):
    """Raised when the specified function is not found in the project."""
    
    def __init__(self, function_name: str, project_path: str):
        self.function_name = function_name
        self.project_path = project_path
        super().__init__(f"Function '{function_name}' not found in project at {project_path}")


class BinaryNotFoundError(AnalyzerError):
    """Raised when the analyzer binary is not found or cannot be executed."""
    
    def __init__(self, binary_path: str):
        self.binary_path = binary_path
        super().__init__(f"Analyzer binary not found at: {binary_path}")


class AnalysisFailedError(AnalyzerError):
    """Raised when the analysis process fails for any reason."""
    
    def __init__(self, message: str, return_code: int = None):
        self.return_code = return_code
        if return_code is not None:
            message = f"{message} (exit code: {return_code})"
        super().__init__(message)