"""
Move Function Analyzer - A Python library for analyzing Move functions in Sui Move projects.

This library provides a simple interface to analyze Move functions and extract detailed
information including source code, parameters, location information, and function call relationships.
"""

from .analyzer import MoveFunctionAnalyzer, AnalysisResult, FunctionCall, Parameter, LocationInfo
from .exceptions import AnalyzerError, ProjectNotFoundError, FunctionNotFoundError, BinaryNotFoundError

__version__ = "1.0.0"
__author__ = "Move Contributors"
__email__ = "opensource@movebit.xyz"

__all__ = [
    "MoveFunctionAnalyzer",
    "AnalysisResult", 
    "FunctionCall",
    "Parameter",
    "LocationInfo",
    "AnalyzerError",
    "ProjectNotFoundError", 
    "FunctionNotFoundError",
    "BinaryNotFoundError",
]