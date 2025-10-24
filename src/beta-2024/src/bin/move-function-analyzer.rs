// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

//! Move Function Analyzer Command Line Tool
//! 
//! This tool provides a command-line interface for analyzing Move functions in Sui Move projects.
//! It allows users to specify a project path and function name to get detailed analysis including
//! source code, parameters, location information, and function call relationships.

use beta_2024::function_analyzer::{FunctionAnalyzer, AnalyzerError};
use clap::{Arg, Command, ArgMatches};
use serde_json;
use std::path::PathBuf;
use std::process;

/// Main entry point for the command line tool
fn main() {
    // Parse command line arguments
    let matches = create_cli_app().get_matches();
    
    // Execute the analysis based on command line arguments
    if let Err(_) = run_analysis(&matches) {
        process::exit(1);
    }
}

/// Create the CLI application with all arguments and options
/// 
/// This function defines the command line interface including:
/// - Project path argument (required)
/// - Function name argument (required)
/// - Output format options
/// - Verbosity controls
/// - Help information
/// 
/// # Returns
/// * `Command` - The configured clap Command for argument parsing
/// 
/// # Requirements
/// Addresses requirements 1.1, 2.1, 6.1 from the specification
fn create_cli_app() -> Command {
    Command::new("move-function-analyzer")
        .version("1.0.0")
        .author("Move Contributors")
        .about("Analyze Move functions in Sui Move projects")
        .long_about(
            "Move Function Analyzer is a tool for analyzing Move functions in Sui Move projects.\n\
            It extracts detailed information about functions including source code, parameters,\n\
            location information, and function call relationships.\n\n\
            The tool outputs results in JSON format for easy integration with other tools."
        )
        .arg(
            Arg::new("project-path")
                .value_name("PROJECT_PATH")
                .help("Path to the Sui Move project directory (containing Move.toml)")
                .long_help(
                    "Path to the Sui Move project directory that contains the Move.toml file.\n\
                    The tool will load and analyze all Move source files in this project."
                )
                .required(true)
                .index(1)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("function-name")
                .value_name("FUNCTION_NAME")
                .help("Name of the function to analyze")
                .long_help(
                    "Name of the function to analyze. The tool will search for all\n\
                    functions with this name across all modules in the project and return detailed\n\
                    analysis for each match."
                )
                .required(true)
                .index(2)
        )



        .after_help(
            "EXAMPLES:\n    \
            move-function-analyzer ./my-project transfer\n    \
            move-function-analyzer /path/to/project mint\n    \
            move-function-analyzer . \"public_transfer\"\n\n\
            OUTPUT:\n    \
            The tool outputs formatted JSON containing function analysis results. Each result includes:\n    \
            - contract: Module name containing the function\n    \
            - function: Function signature with parameters and return type\n    \
            - source: Complete source code of the function\n    \
            - location: File path and line numbers\n    \
            - parameter: List of parameter names and types\n    \
            - calls: List of functions called by this function"
        )
}

/// Run the function analysis based on command line arguments
/// 
/// This function coordinates the entire analysis process:
/// 1. Extracts arguments from command line
/// 2. Initializes the function analyzer
/// 3. Performs the analysis
/// 4. Formats and outputs the results
/// 
/// # Arguments
/// * `matches` - Parsed command line arguments
/// 
/// # Returns
/// * `Result<(), AnalyzerError>` - Success or detailed error information
/// 
/// # Requirements
/// Addresses requirements 1.1, 2.1, 6.1 from the specification
fn run_analysis(matches: &ArgMatches) -> Result<(), AnalyzerError> {
    // Extract command line arguments
    let project_path = matches.get_one::<PathBuf>("project-path")
        .expect("project-path is required")
        .clone();
    
    let function_name = matches.get_one::<String>("function-name")
        .expect("function-name is required");
    
    // Validate project path exists
    if !project_path.exists() {
        return Err(AnalyzerError::InvalidProjectPath(project_path));
    }
    
    // Initialize the function analyzer
    let analyzer = FunctionAnalyzer::new(project_path.clone())?;
    
    // Perform the function analysis
    let results = analyzer.analyze_function(function_name)?;
    
    // Output the results (empty array if no functions found)
    output_results(&results)?;
    
    Ok(())
}



/// Output analysis results in JSON format
/// 
/// This function formats and outputs the analysis results as JSON
/// with pretty-printing for better readability.
/// 
/// # Arguments
/// * `results` - Vector of function analysis results
/// 
/// # Returns
/// * `Result<(), AnalyzerError>` - Success or JSON serialization error
/// 
/// # Requirements
/// Addresses requirements 6.1, 6.2, 6.3 from the specification
fn output_results(
    results: &[beta_2024::function_analyzer::FunctionAnalysis]
) -> Result<(), AnalyzerError> {
    let json_output = serde_json::to_string_pretty(results)?;
    println!("{}", json_output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    /// Test CLI argument parsing with valid arguments
    #[test]
    fn test_cli_parsing_valid_args() {
        let app = create_cli_app();
        let matches = app.try_get_matches_from(vec![
            "move-function-analyzer",
            "/tmp/test",
            "test_function"
        ]).unwrap();
        
        assert_eq!(
            matches.get_one::<PathBuf>("project-path").unwrap(),
            &PathBuf::from("/tmp/test")
        );
        assert_eq!(
            matches.get_one::<String>("function-name").unwrap(),
            "test_function"
        );

    }

    /// Test CLI argument parsing with all options
    #[test]
    fn test_cli_parsing_all_options() {
        let app = create_cli_app();
        let matches = app.try_get_matches_from(vec![
            "move-function-analyzer",
            ".",
            "mint"
        ]).unwrap();
        
        assert_eq!(
            matches.get_one::<PathBuf>("project-path").unwrap(),
            &PathBuf::from(".")
        );
        assert_eq!(
            matches.get_one::<String>("function-name").unwrap(),
            "mint"
        );
    }

    /// Test CLI argument parsing with missing required arguments
    #[test]
    fn test_cli_parsing_missing_args() {
        let app = create_cli_app();
        
        // Missing function name
        let result = app.try_get_matches_from(vec![
            "move-function-analyzer",
            "/tmp/test"
        ]);
        assert!(result.is_err());
        
        // Missing project path
        let app = create_cli_app();
        let result = app.try_get_matches_from(vec![
            "move-function-analyzer"
        ]);
        assert!(result.is_err());
    }



    /// Test output formatting with empty results
    #[test]
    fn test_output_empty_results() {
        let results = vec![];
        
        // Test output
        let result = output_results(&results);
        assert!(result.is_ok());
    }


}