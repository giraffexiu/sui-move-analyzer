// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

//! Move Function Analyzer
//! 
//! This module provides functionality to analyze Move functions, extracting detailed information
//! including source code, parameters, location information, and function call relationships.

use crate::{project::Project, project_context::ProjectContext, context::MultiProject};
use move_compiler::parser::ast::{Function, Type, Type_};
use move_ir_types::location::Loc;
use move_core_types::account_address::AccountAddress;
use move_symbol_pool::Symbol;
use move_package::source_package::parsed_manifest::SourceManifest;
use move_package::source_package::manifest_parser::parse_move_manifest_from_file;
use move_compiler::editions::Edition;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir;

/// Main result structure containing comprehensive function analysis information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionAnalysis {
    /// The contract/module name containing the function
    pub contract: String,
    /// The function signature including name, parameters, and return type
    pub function: String,
    /// The complete source code of the function
    pub source: String,
    /// Location information including file path and line numbers
    pub location: LocationInfo,
    /// List of function parameters with names and types
    #[serde(rename = "parameter")]
    pub parameters: Vec<Parameter>,
    /// List of functions called by this function
    pub calls: Vec<FunctionCall>,
}

/// Location information for a function in the source code
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocationInfo {
    /// Path to the source file containing the function
    pub file: PathBuf,
    /// Starting line number of the function (1-indexed)
    pub start_line: u32,
    /// Ending line number of the function (1-indexed)
    pub end_line: u32,
}

/// Information about a function parameter
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type as a string representation
    #[serde(rename = "type")]
    pub type_: String,
}

/// Information about a function call made within the analyzed function
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// Path to the file containing the called function
    pub file: PathBuf,
    /// Function signature of the called function
    pub function: String,
    /// Module name containing the called function
    pub module: String,
}

/// Error types that can occur during function analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    /// Invalid or non-existent project path
    #[error("Invalid project path: {0}")]
    InvalidProjectPath(PathBuf),
    
    /// Move.toml file is missing or invalid
    #[error("Move.toml file not found or invalid")]
    InvalidMoveToml,
    
    /// Specified function was not found in the project
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    
    /// Error occurred during parsing of Move source code
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Error occurred during type resolution
    #[error("Type resolution error: {0}")]
    TypeResolutionError(String),
    
    /// IO error occurred during file operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Generic error for other issues
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

/// Result type for function analysis operations
pub type AnalyzerResult<T> = Result<T, AnalyzerError>;

/// Project loader for loading and validating Sui Move projects
pub struct ProjectLoader;

impl ProjectLoader {
    /// Load a Sui Move project from the given directory path
    /// 
    /// This method validates the project structure, parses the Move.toml file,
    /// and creates a Project instance with all necessary context.
    /// 
    /// # Arguments
    /// * `project_path` - Path to the directory containing the Move.toml file
    /// 
    /// # Returns
    /// * `AnalyzerResult<Project>` - The loaded project or an error
    /// 
    /// # Requirements
    /// Addresses requirements 1.1, 1.2, 1.3 from the specification
    pub fn load_project(project_path: PathBuf) -> AnalyzerResult<Project> {
        // Validate the project path and structure
        Self::validate_move_project(&project_path)?;
        
        // Parse the Move.toml file to ensure it's valid
        let _manifest = Self::parse_move_toml(&project_path)?;
        
        // Create a MultiProject context for loading
        let mut multi_project = MultiProject::new();
        
        // Use the existing Project::new functionality with implicit dependencies
        let implicit_deps = crate::implicit_deps();
        
        // Create error reporter that collects errors
        let errors = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let errors_clone = errors.clone();
        let error_reporter = move |error: String| {
            errors_clone.borrow_mut().push(error);
        };
        
        // Load the project using the existing infrastructure
        let project = Project::new(
            project_path.clone(),
            &mut multi_project,
            error_reporter,
            implicit_deps,
        ).map_err(|e| AnalyzerError::AnalysisError(format!("Failed to load project: {}", e)))?;
        
        // Check if there were any errors during loading
        let errors_vec = errors.borrow();
        if !errors_vec.is_empty() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Project loading errors: {}",
                errors_vec.join("; ")
            )));
        }
        
        // Note: We allow projects that don't fully load (load_ok() == false) to proceed
        // This can happen when external dependencies are not available, but we can still
        // analyze the source code that was successfully parsed.
        if !project.load_ok() {
            log::warn!("Project did not load completely - some dependencies may be missing, but proceeding with available source code");
        }
        
        Ok(project)
    }
    
    /// Validate that the given path contains a valid Move project structure
    /// 
    /// This method performs comprehensive validation of the project structure,
    /// including Move.toml existence and validity, directory structure integrity,
    /// and basic accessibility checks.
    /// 
    /// # Arguments
    /// * `project_path` - Path to validate
    /// 
    /// # Returns
    /// * `AnalyzerResult<()>` - Success or detailed validation error
    /// 
    /// # Requirements
    /// Addresses requirements 1.1, 1.4 from the specification
    fn validate_move_project(project_path: &Path) -> AnalyzerResult<()> {
        // Check if the path exists and is accessible
        if !project_path.exists() {
            return Err(AnalyzerError::InvalidProjectPath(project_path.to_path_buf()));
        }
        
        if !project_path.is_dir() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Project path is not a directory: {}",
                project_path.display()
            )));
        }
        
        // Check if we can read the directory
        match fs::read_dir(project_path) {
            Ok(_) => {},
            Err(e) => {
                return Err(AnalyzerError::AnalysisError(format!(
                    "Cannot read project directory {}: {}",
                    project_path.display(),
                    e
                )));
            }
        }
        
        // Check for Move.toml file existence and accessibility
        let move_toml_path = project_path.join("Move.toml");
        if !move_toml_path.exists() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Move.toml file not found in project directory: {}",
                project_path.display()
            )));
        }
        
        if !move_toml_path.is_file() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Move.toml exists but is not a file: {}",
                move_toml_path.display()
            )));
        }
        
        // Check if Move.toml is readable
        match fs::read_to_string(&move_toml_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Err(AnalyzerError::AnalysisError(
                        "Move.toml file is empty".to_string()
                    ));
                }
            },
            Err(e) => {
                return Err(AnalyzerError::AnalysisError(format!(
                    "Cannot read Move.toml file: {}",
                    e
                )));
            }
        }
        
        // Validate directory structure integrity
        Self::validate_directory_structure(project_path)?;
        
        Ok(())
    }
    
    /// Validate the integrity of the project directory structure
    /// 
    /// This method checks for common Move project directories and ensures
    /// they are properly structured if they exist.
    /// 
    /// # Arguments
    /// * `project_path` - Path to the project directory
    /// 
    /// # Returns
    /// * `AnalyzerResult<()>` - Success or validation error
    /// 
    /// # Requirements
    /// Addresses requirements 1.1, 1.4 from the specification
    fn validate_directory_structure(project_path: &Path) -> AnalyzerResult<()> {
        // Check sources directory (optional but common)
        let sources_path = project_path.join("sources");
        if sources_path.exists() {
            if !sources_path.is_dir() {
                return Err(AnalyzerError::AnalysisError(format!(
                    "sources path exists but is not a directory: {}",
                    sources_path.display()
                )));
            }
            
            // Check if sources directory is readable
            match fs::read_dir(&sources_path) {
                Ok(_) => {},
                Err(e) => {
                    return Err(AnalyzerError::AnalysisError(format!(
                        "Cannot read sources directory: {}",
                        e
                    )));
                }
            }
            
            // Check if there are any .move files in sources
            Self::validate_move_files_in_directory(&sources_path, "sources")?;
        }
        
        // Check tests directory (optional)
        let tests_path = project_path.join("tests");
        if tests_path.exists() {
            if !tests_path.is_dir() {
                return Err(AnalyzerError::AnalysisError(format!(
                    "tests path exists but is not a directory: {}",
                    tests_path.display()
                )));
            }
            
            // Check if tests directory is readable
            match fs::read_dir(&tests_path) {
                Ok(_) => {},
                Err(e) => {
                    return Err(AnalyzerError::AnalysisError(format!(
                        "Cannot read tests directory: {}",
                        e
                    )));
                }
            }
        }
        
        // Check scripts directory (optional)
        let scripts_path = project_path.join("scripts");
        if scripts_path.exists() {
            if !scripts_path.is_dir() {
                return Err(AnalyzerError::AnalysisError(format!(
                    "scripts path exists but is not a directory: {}",
                    scripts_path.display()
                )));
            }
            
            // Check if scripts directory is readable
            match fs::read_dir(&scripts_path) {
                Ok(_) => {},
                Err(e) => {
                    return Err(AnalyzerError::AnalysisError(format!(
                        "Cannot read scripts directory: {}",
                        e
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate Move files in a given directory
    /// 
    /// This method checks if there are readable .move files in the directory
    /// and performs basic validation on them.
    /// 
    /// # Arguments
    /// * `dir_path` - Path to the directory to check
    /// * `dir_name` - Name of the directory for error messages
    /// 
    /// # Returns
    /// * `AnalyzerResult<()>` - Success or validation error
    fn validate_move_files_in_directory(dir_path: &Path, dir_name: &str) -> AnalyzerResult<()> {
        let mut has_move_files = false;
        let mut validation_errors = Vec::new();
        
        // Walk through the directory recursively
        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    validation_errors.push(format!(
                        "Error reading {} directory: {}",
                        dir_name, e
                    ));
                    continue;
                }
            };
            
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".move") && !file_name.starts_with('.') {
                        has_move_files = true;
                        
                        // Check if the file is readable
                        match fs::read_to_string(entry.path()) {
                            Ok(content) => {
                                if content.trim().is_empty() {
                                    validation_errors.push(format!(
                                        "Move file is empty: {}",
                                        entry.path().display()
                                    ));
                                }
                            },
                            Err(e) => {
                                validation_errors.push(format!(
                                    "Cannot read Move file {}: {}",
                                    entry.path().display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        // Report any validation errors found
        if !validation_errors.is_empty() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Validation errors in {} directory: {}",
                dir_name,
                validation_errors.join("; ")
            )));
        }
        
        // Note: It's okay to have no .move files in some directories (like tests)
        // so we don't treat this as an error, just log it for debugging
        if !has_move_files {
            log::debug!("No .move files found in {} directory: {}", dir_name, dir_path.display());
        }
        
        Ok(())
    }
    
    /// Parse and validate the Move.toml manifest file
    /// 
    /// This method attempts to parse the Move.toml file to ensure it's valid
    /// and contains the necessary project configuration. It performs comprehensive
    /// validation of the manifest structure and content.
    /// 
    /// # Arguments
    /// * `project_path` - Path to the project directory
    /// 
    /// # Returns
    /// * `AnalyzerResult<SourceManifest>` - The parsed manifest or detailed error
    /// 
    /// # Requirements
    /// Addresses requirements 1.1, 1.4 from the specification
    fn parse_move_toml(project_path: &Path) -> AnalyzerResult<SourceManifest> {
        let move_toml_path = project_path.join("Move.toml");
        
        // Parse the manifest file
        let manifest = parse_move_manifest_from_file(project_path)
            .map_err(|e| {
                AnalyzerError::ParseError(format!(
                    "Failed to parse Move.toml at {}: {}",
                    move_toml_path.display(),
                    e
                ))
            })?;
        
        // Validate the manifest content
        Self::validate_manifest_content(&manifest)?;
        
        Ok(manifest)
    }
    
    /// Validate the content of a parsed Move.toml manifest
    /// 
    /// This method performs detailed validation of the manifest structure
    /// and ensures all required fields are present and valid.
    /// 
    /// # Arguments
    /// * `manifest` - The parsed manifest to validate
    /// 
    /// # Returns
    /// * `AnalyzerResult<()>` - Success or validation error
    /// 
    /// # Requirements
    /// Addresses requirements 1.1, 1.4 from the specification
    fn validate_manifest_content(manifest: &SourceManifest) -> AnalyzerResult<()> {
        // Validate package section
        if manifest.package.name.as_str().is_empty() {
            return Err(AnalyzerError::ParseError(
                "Package name cannot be empty in Move.toml".to_string()
            ));
        }
        
        // Validate package name format (basic check)
        let package_name = manifest.package.name.as_str();
        if !package_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AnalyzerError::ParseError(format!(
                "Invalid package name '{}': must contain only alphanumeric characters, underscores, and hyphens",
                package_name
            )));
        }
        
        // Validate edition if present
        if let Some(edition) = &manifest.package.edition {
            let edition_str = match edition {
                &Edition::E2024_BETA => "2024.beta",
                &Edition::E2024_ALPHA => "2024.alpha",
                _ => "unknown", // Handle any other variants
            };
            // Edition validation is implicit in the enum, so we just log it
            log::debug!("Project uses edition: {}", edition_str);
        }
        
        // Validate addresses section if present
        if let Some(ref addresses) = manifest.addresses {
            for (name, address_opt) in addresses {
                if name.as_str().is_empty() {
                    return Err(AnalyzerError::ParseError(
                        "Address name cannot be empty".to_string()
                    ));
                }
                
                // If address is specified (not a placeholder), validate it
                if let Some(address) = address_opt {
                    // Basic validation - addresses should be valid
                    let address_str = format!("{}", address);
                    if address_str.is_empty() {
                        return Err(AnalyzerError::ParseError(format!(
                            "Invalid address for '{}': address cannot be empty",
                            name.as_str()
                        )));
                    }
                }
            }
        }
        
        // Validate dev-addresses section if present
        if let Some(ref dev_addresses) = manifest.dev_address_assignments {
            for (name, address) in dev_addresses {
                if name.as_str().is_empty() {
                    return Err(AnalyzerError::ParseError(
                        "Dev address name cannot be empty".to_string()
                    ));
                }
                
                let address_str = format!("{}", address);
                if address_str.is_empty() {
                    return Err(AnalyzerError::ParseError(format!(
                        "Invalid dev address for '{}': address cannot be empty",
                        name.as_str()
                    )));
                }
            }
        }
        
        // Validate dependencies section - it's a BTreeMap, not Option
        for (dep_name, _dependency) in &manifest.dependencies {
            if dep_name.as_str().is_empty() {
                return Err(AnalyzerError::ParseError(
                    "Dependency name cannot be empty".to_string()
                ));
            }
        }
        
        // Validate dev-dependencies section - it's a BTreeMap, not Option
        for (dep_name, _dependency) in &manifest.dev_dependencies {
            if dep_name.as_str().is_empty() {
                return Err(AnalyzerError::ParseError(
                    "Dev dependency name cannot be empty".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

/// Information about a function definition found in the project
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    /// The AST function definition
    pub function: Function,
    /// Module information containing this function
    pub module_info: ModuleInfo,
    /// Location in source code
    pub location: Loc,
}

/// Information about a module containing functions
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleInfo {
    /// Module address
    pub address: AccountAddress,
    /// Module name
    pub name: Symbol,
    /// File path containing the module
    pub file_path: PathBuf,
}

/// Type resolver for converting Move AST types to readable string representations
/// 
/// This resolver handles all Move type constructs including basic types, references,
/// generics, structs, and complex nested types. It provides comprehensive support
/// for Move's type system as specified in requirements 4.1, 4.2, 4.4, and 7.3.
pub struct TypeResolver<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> TypeResolver<'a> {
    /// Create a new TypeResolver instance
    /// 
    /// # Arguments
    /// * `project` - Reference to the loaded Move project
    /// * `context` - Reference to the project context for symbol resolution
    /// 
    /// # Returns
    /// * `TypeResolver` - New resolver instance
    /// 
    /// # Requirements
    /// Addresses requirements 4.1, 4.2, 4.4, 7.3 from the specification
    pub fn new(_project: &'a Project, _context: &'a ProjectContext) -> Self {
        Self { _phantom: std::marker::PhantomData }
    }

    /// Convert a Move AST type to its string representation
    /// 
    /// This method handles all Move type constructs and produces readable
    /// string representations that match Move's syntax conventions.
    /// 
    /// # Arguments
    /// * `type_` - The Move AST type to convert
    /// 
    /// # Returns
    /// * `String` - Human-readable type string representation
    /// 
    /// # Requirements
    /// Addresses requirements 4.1, 4.2, 4.4, 7.3 from the specification
    pub fn type_to_string(&self, type_: &Type) -> String {
        match &type_.value {
            Type_::Unit => self.format_unit_type(),
            Type_::Apply(name_access_chain) => self.format_apply_type(name_access_chain),
            Type_::Ref(is_mut, inner_type) => self.format_reference_type(*is_mut, inner_type),
            Type_::Fun(params, return_type) => self.format_function_type(params, return_type),
            Type_::Multiple(types) => self.format_multiple_type(types),
            Type_::UnresolvedError => self.format_unresolved_error(),
        }
    }

    /// Format unit type ()
    /// 
    /// # Returns
    /// * `String` - The unit type representation "()"
    /// 
    /// # Requirements
    /// Addresses requirement 4.1 - basic Move types
    fn format_unit_type(&self) -> String {
        "()".to_string()
    }

    /// Format applied types (basic types, structs, generics)
    /// 
    /// This handles basic Move types (u8, u64, bool, address, etc.),
    /// struct types, and generic type applications.
    /// 
    /// # Arguments
    /// * `name_access_chain` - The name access chain representing the type
    /// 
    /// # Returns
    /// * `String` - Formatted type string
    /// 
    /// # Requirements
    /// Addresses requirements 4.1, 4.2, 4.4 - basic types, complex types, generics
    fn format_apply_type(&self, name_access_chain: &move_compiler::parser::ast::NameAccessChain) -> String {
        self.name_access_chain_to_string(name_access_chain)
    }

    /// Format reference types (&T, &mut T)
    /// 
    /// Handles both immutable and mutable references according to Move syntax.
    /// 
    /// # Arguments
    /// * `is_mut` - Whether this is a mutable reference
    /// * `inner_type` - The referenced type
    /// 
    /// # Returns
    /// * `String` - Formatted reference type string
    /// 
    /// # Requirements
    /// Addresses requirement 4.1 - reference types (&T, &mut T)
    fn format_reference_type(&self, is_mut: bool, inner_type: &Type) -> String {
        let mut_str = if is_mut { "mut " } else { "" };
        format!("&{}{}", mut_str, self.type_to_string(inner_type))
    }

    /// Format function types (lambda/closure types)
    /// 
    /// Handles function type syntax |param_types| -> return_type
    /// 
    /// # Arguments
    /// * `params` - Parameter types of the function
    /// * `return_type` - Return type of the function
    /// 
    /// # Returns
    /// * `String` - Formatted function type string
    /// 
    /// # Requirements
    /// Addresses requirement 4.2 - complex type handling
    fn format_function_type(&self, params: &[Type], return_type: &Type) -> String {
        let param_strings: Vec<String> = params.iter()
            .map(|t| self.type_to_string(t))
            .collect();
        format!("|{}| -> {}", param_strings.join(", "), self.type_to_string(return_type))
    }

    /// Format multiple/tuple types (T1, T2, ..., Tn)
    /// 
    /// Handles tuple types and multiple return values.
    /// 
    /// # Arguments
    /// * `types` - Vector of types in the tuple
    /// 
    /// # Returns
    /// * `String` - Formatted tuple type string
    /// 
    /// # Requirements
    /// Addresses requirement 4.2 - complex type handling
    fn format_multiple_type(&self, types: &[Type]) -> String {
        let type_strings: Vec<String> = types.iter()
            .map(|t| self.type_to_string(t))
            .collect();
        format!("({})", type_strings.join(", "))
    }

    /// Format unresolved error types
    /// 
    /// # Returns
    /// * `String` - Error type representation
    fn format_unresolved_error(&self) -> String {
        "UnresolvedError".to_string()
    }

    /// Convert a name access chain to its string representation
    /// 
    /// This method handles various forms of type names including:
    /// - Basic types: u8, u64, bool, address, signer
    /// - Qualified types: Module::Type
    /// - Generic types: Type<T1, T2>
    /// - Nested module access: Address::Module::Type
    /// 
    /// # Arguments
    /// * `name_access_chain` - The name access chain to convert
    /// 
    /// # Returns
    /// * `String` - String representation of the type name
    /// 
    /// # Requirements
    /// Addresses requirements 4.1, 4.2, 4.4, 7.3 - all type representations
    fn name_access_chain_to_string(&self, name_access_chain: &move_compiler::parser::ast::NameAccessChain) -> String {
        use move_compiler::parser::ast::NameAccessChain_ as NAC;
        
        match &name_access_chain.value {
            NAC::Single(path_entry) => {
                self.format_single_name_entry(path_entry)
            }
            NAC::Path(name_path) => {
                self.format_path_name_access(name_path)
            }
        }
    }

    /// Format a single name entry (basic type or unqualified name)
    /// 
    /// This method handles individual path entries in name access chains,
    /// including type arguments for generic types.
    /// 
    /// # Arguments
    /// * `path_entry` - The path entry to format
    /// 
    /// # Returns
    /// * `String` - Formatted name string
    fn format_single_name_entry(&self, path_entry: &move_compiler::parser::ast::PathEntry) -> String {
        let name = path_entry.name.value.as_str();
        
        // Handle basic Move types
        if self.is_basic_move_type(name) {
            return name.to_string();
        }
        
        // Handle type arguments if present
        if let Some(type_args) = &path_entry.tyargs {
            let type_arg_strings: Vec<String> = type_args.value.iter()
                .map(|t| self.type_to_string(t))
                .collect();
            format!("{}<{}>", name, type_arg_strings.join(", "))
        } else {
            name.to_string()
        }
    }

    /// Format a path-based name access (qualified names)
    /// 
    /// This method handles qualified names like Module::Type or Address::Module::Type.
    /// For now, it uses a simplified approach similar to the existing module_access_to_string.
    /// 
    /// # Arguments
    /// * `name_path` - The name path to format
    /// 
    /// # Returns
    /// * `String` - Formatted qualified name string
    fn format_path_name_access(&self, name_path: &move_compiler::parser::ast::NamePath) -> String {
        // Use a simplified approach similar to the existing working code
        match &name_path.root.name.value {
            move_compiler::parser::ast::LeadingNameAccess_::Name(name) => {
                // For now, just return the name - this could be enhanced later
                name.value.as_str().to_string()
            }
            move_compiler::parser::ast::LeadingNameAccess_::GlobalAddress(name) => {
                format!("@{}", name.value.as_str())
            }
            move_compiler::parser::ast::LeadingNameAccess_::AnonymousAddress(addr) => {
                format!("0x{}", hex::encode(addr.into_bytes()))
            }
        }
    }

    /// Check if a name represents a basic Move type
    /// 
    /// # Arguments
    /// * `name` - The type name to check
    /// 
    /// # Returns
    /// * `bool` - True if this is a basic Move type
    /// 
    /// # Requirements
    /// Addresses requirement 4.1 - basic Move types
    fn is_basic_move_type(&self, name: &str) -> bool {
        matches!(name, 
            "u8" | "u16" | "u32" | "u64" | "u128" | "u256" | 
            "bool" | "address" | "signer" | "vector"
        )
    }

    /// Resolve and format a struct type with its full qualification
    /// 
    /// This method attempts to resolve struct types to their fully qualified names
    /// and handles generic type parameters appropriately.
    /// 
    /// # Arguments
    /// * `struct_name` - The struct name to resolve
    /// * `type_args` - Optional type arguments for generic structs
    /// 
    /// # Returns
    /// * `String` - Fully qualified struct type string
    /// 
    /// # Requirements
    /// Addresses requirements 4.2, 4.4 - struct types and generics
    pub fn resolve_struct_type(&self, struct_name: &str, type_args: Option<&[Type]>) -> String {
        // Try to find the struct in the current project context
        let qualified_name = self.find_qualified_struct_name(struct_name)
            .unwrap_or_else(|| struct_name.to_string());
        
        if let Some(args) = type_args {
            if !args.is_empty() {
                let type_arg_strings: Vec<String> = args.iter()
                    .map(|t| self.type_to_string(t))
                    .collect();
                format!("{}<{}>", qualified_name, type_arg_strings.join(", "))
            } else {
                qualified_name
            }
        } else {
            qualified_name
        }
    }

    /// Find the fully qualified name for a struct
    /// 
    /// # Arguments
    /// * `struct_name` - The struct name to look up
    /// 
    /// # Returns
    /// * `Option<String>` - Fully qualified name if found
    fn find_qualified_struct_name(&self, struct_name: &str) -> Option<String> {
        // This is a simplified implementation
        // In a full implementation, we would search through the project context
        // to find the module containing this struct and return the qualified name
        
        // For now, return the struct name as-is
        // This could be enhanced to search through project modules
        Some(struct_name.to_string())
    }

    /// Format a vector type with its element type
    /// 
    /// # Arguments
    /// * `element_type` - The type of elements in the vector
    /// 
    /// # Returns
    /// * `String` - Formatted vector type string
    /// 
    /// # Requirements
    /// Addresses requirement 4.2 - complex type handling (vectors)
    pub fn format_vector_type(&self, element_type: &Type) -> String {
        format!("vector<{}>", self.type_to_string(element_type))
    }

    /// Check if a type is a Move resource type
    /// 
    /// This method determines if a given type represents a Move resource,
    /// which has special semantics in the Move language.
    /// 
    /// # Arguments
    /// * `type_name` - The type name to check
    /// 
    /// # Returns
    /// * `bool` - True if this is likely a resource type
    /// 
    /// # Requirements
    /// Addresses requirement 7.3 - Move resource types
    pub fn is_resource_type(&self, type_name: &str) -> bool {
        // In Move, resource types are typically structs that don't have copy/drop abilities
        // This is a simplified check - a full implementation would examine the struct definition
        
        // Common resource type patterns in Sui Move
        type_name.contains("Coin") || 
        type_name.contains("Object") || 
        type_name.contains("UID") ||
        type_name.ends_with("Cap") ||
        type_name.ends_with("Witness")
    }

    /// Generate a readable type string representation
    /// 
    /// This method produces human-readable type strings that are suitable
    /// for display in documentation, error messages, and analysis output.
    /// 
    /// # Arguments
    /// * `type_` - The type to format
    /// 
    /// # Returns
    /// * `String` - Human-readable type representation
    /// 
    /// # Requirements
    /// Addresses requirement 4.2 - readable type string representation
    pub fn generate_readable_type_string(&self, type_: &Type) -> String {
        let base_string = self.type_to_string(type_);
        
        // Add additional formatting for better readability
        self.enhance_type_readability(&base_string)
    }

    /// Enhance type string readability
    /// 
    /// # Arguments
    /// * `type_string` - The base type string
    /// 
    /// # Returns
    /// * `String` - Enhanced readable type string
    fn enhance_type_readability(&self, type_string: &str) -> String {
        // Add spacing around generic brackets for better readability
        let enhanced = type_string
            .replace("<", " <")
            .replace(">", "> ")
            .replace("  ", " ")
            .trim()
            .to_string();
        
        // Remove extra spaces that might have been introduced
        enhanced.replace(" <", "<").replace("> ", ">")
    }

    /// Handle nested generic types with proper formatting
    /// 
    /// This method processes complex nested generic types like
    /// `Option<vector<Table<address, Coin<SUI>>>>` and ensures
    /// proper formatting and readability.
    /// 
    /// # Arguments
    /// * `base_type` - The base type name
    /// * `type_args` - Nested type arguments
    /// 
    /// # Returns
    /// * `String` - Properly formatted nested generic type
    /// 
    /// # Requirements
    /// Addresses requirement 4.2 - nested generic types
    pub fn handle_nested_generics(&self, base_type: &str, type_args: &[Type]) -> String {
        if type_args.is_empty() {
            return base_type.to_string();
        }

        let mut formatted_args = Vec::new();
        
        for type_arg in type_args {
            let arg_string = self.type_to_string(type_arg);
            
            // Handle deeply nested generics by adding proper spacing
            let formatted_arg = if arg_string.contains('<') && arg_string.contains('>') {
                self.format_deeply_nested_type(&arg_string)
            } else {
                arg_string
            };
            
            formatted_args.push(formatted_arg);
        }
        
        format!("{}<{}>", base_type, formatted_args.join(", "))
    }

    /// Format deeply nested generic types for better readability
    /// 
    /// # Arguments
    /// * `type_string` - The nested type string
    /// 
    /// # Returns
    /// * `String` - Formatted nested type with proper spacing
    fn format_deeply_nested_type(&self, type_string: &str) -> String {
        // Count nesting depth and add appropriate formatting
        let depth = type_string.matches('<').count();
        
        if depth > 2 {
            // For very deep nesting, consider line breaks or simplified representation
            self.simplify_deep_nesting(type_string)
        } else {
            type_string.to_string()
        }
    }

    /// Simplify deeply nested types for readability
    /// 
    /// # Arguments
    /// * `type_string` - The deeply nested type string
    /// 
    /// # Returns
    /// * `String` - Simplified type representation
    fn simplify_deep_nesting(&self, type_string: &str) -> String {
        // For very complex types, we might want to truncate or simplify
        // For now, just return the original string
        // In a production system, this could be configurable
        type_string.to_string()
    }

    /// Resolve Move resource types with their capabilities
    /// 
    /// This method identifies Move resource types and provides information
    /// about their capabilities (copy, drop, store, key).
    /// 
    /// # Arguments
    /// * `type_name` - The type name to analyze
    /// 
    /// # Returns
    /// * `(String, Vec<String>)` - Type name and list of capabilities
    /// 
    /// # Requirements
    /// Addresses requirement 7.3 - Move resource types
    pub fn resolve_resource_type_with_capabilities(&self, type_name: &str) -> (String, Vec<String>) {
        let capabilities = self.infer_type_capabilities(type_name);
        (type_name.to_string(), capabilities)
    }

    /// Infer the capabilities of a Move type
    /// 
    /// # Arguments
    /// * `type_name` - The type name to analyze
    /// 
    /// # Returns
    /// * `Vec<String>` - List of inferred capabilities
    fn infer_type_capabilities(&self, type_name: &str) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Basic types have copy + drop + store
        if self.is_basic_move_type(type_name) {
            capabilities.extend_from_slice(&["copy".to_string(), "drop".to_string(), "store".to_string()]);
        }
        // Resource types typically don't have copy/drop
        else if self.is_resource_type(type_name) {
            capabilities.push("key".to_string());
            if type_name.contains("Store") || type_name.ends_with("Data") {
                capabilities.push("store".to_string());
            }
        }
        // Vector types inherit capabilities from their element type
        else if type_name.starts_with("vector<") {
            capabilities.extend_from_slice(&["store".to_string()]);
        }
        // Default for unknown types
        else {
            capabilities.push("unknown".to_string());
        }
        
        capabilities
    }

    /// Generate comprehensive type information including metadata
    /// 
    /// This method produces detailed type information suitable for
    /// analysis tools and IDE features.
    /// 
    /// # Arguments
    /// * `type_` - The type to analyze
    /// 
    /// # Returns
    /// * `TypeInfo` - Comprehensive type information
    /// 
    /// # Requirements
    /// Addresses requirement 4.2 - readable type string representation
    pub fn generate_comprehensive_type_info(&self, type_: &Type) -> TypeInfo {
        let type_string = self.type_to_string(type_);
        let is_reference = matches!(&type_.value, Type_::Ref(_, _));
        let is_mutable = if let Type_::Ref(is_mut, _) = &type_.value { *is_mut } else { false };
        let is_generic = self.contains_generics(&type_string);
        let complexity_level = self.calculate_type_complexity(type_);
        
        TypeInfo {
            type_string: type_string.clone(),
            readable_string: self.generate_readable_type_string(type_),
            is_reference,
            is_mutable,
            is_generic,
            complexity_level,
            capabilities: if is_reference { 
                vec![] 
            } else { 
                self.infer_type_capabilities(&type_string) 
            },
        }
    }

    /// Check if a type string contains generic parameters
    /// 
    /// # Arguments
    /// * `type_string` - The type string to check
    /// 
    /// # Returns
    /// * `bool` - True if the type contains generics
    fn contains_generics(&self, type_string: &str) -> bool {
        type_string.contains('<') && type_string.contains('>')
    }

    /// Calculate the complexity level of a type
    /// 
    /// # Arguments
    /// * `type_` - The type to analyze
    /// 
    /// # Returns
    /// * `u32` - Complexity level (0 = simple, higher = more complex)
    fn calculate_type_complexity(&self, type_: &Type) -> u32 {
        match &type_.value {
            Type_::Unit => 0,
            Type_::Apply(name_access) => {
                // Count generic nesting depth
                self.calculate_name_access_complexity(name_access)
            }
            Type_::Ref(_, inner) => 1 + self.calculate_type_complexity(inner),
            Type_::Fun(params, ret) => {
                let param_complexity: u32 = params.iter()
                    .map(|p| self.calculate_type_complexity(p))
                    .sum();
                2 + param_complexity + self.calculate_type_complexity(ret)
            }
            Type_::Multiple(types) => {
                let total_complexity: u32 = types.iter()
                    .map(|t| self.calculate_type_complexity(t))
                    .sum();
                1 + total_complexity
            }
            Type_::UnresolvedError => 0,
        }
    }

    /// Calculate complexity of a name access chain
    /// 
    /// # Arguments
    /// * `name_access` - The name access chain to analyze
    /// 
    /// # Returns
    /// * `u32` - Complexity level
    fn calculate_name_access_complexity(&self, name_access: &move_compiler::parser::ast::NameAccessChain) -> u32 {
        use move_compiler::parser::ast::NameAccessChain_ as NAC;
        
        match &name_access.value {
            NAC::Single(path_entry) => {
                if let Some(type_args) = &path_entry.tyargs {
                    let args_complexity: u32 = type_args.value.iter()
                        .map(|t| self.calculate_type_complexity(t))
                        .sum();
                    1 + args_complexity
                } else {
                    1
                }
            }
            NAC::Path(_name_path) => {
                // Simplified complexity calculation for path access
                // In a full implementation, this would traverse the entire path
                2 // Base complexity for qualified names
            }
        }
    }
}

/// Comprehensive type information structure
/// 
/// This structure contains detailed information about a Move type,
/// including its string representation, metadata, and capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    /// The canonical type string representation
    pub type_string: String,
    /// Human-readable type string with enhanced formatting
    pub readable_string: String,
    /// Whether this is a reference type
    pub is_reference: bool,
    /// Whether this is a mutable reference (only meaningful if is_reference is true)
    pub is_mutable: bool,
    /// Whether this type contains generic parameters
    pub is_generic: bool,
    /// Complexity level of the type (0 = simple, higher = more complex)
    pub complexity_level: u32,
    /// Inferred capabilities of the type
    pub capabilities: Vec<String>,
}

/// Function visibility levels in Move
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionVisibility {
    /// Public function accessible from anywhere
    Public,
    /// Public function accessible only to friend modules
    PublicFriend,
    /// Private function accessible only within the same module
    Private,
}

/// Function categories based on modifiers and visibility
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCategory {
    /// Public function
    Public,
    /// Public friend function
    PublicFriend,
    /// Private function
    Private,
    /// Entry function (can be called from transactions)
    Entry,
    /// Native function (implemented in the runtime)
    Native,
}

/// Comprehensive information about a function's type and characteristics
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTypeInfo {
    /// Function visibility level
    pub visibility: FunctionVisibility,
    /// Whether the function has the entry modifier
    pub is_entry: bool,
    /// Whether the function is native
    pub is_native: bool,
    /// Function category combining visibility and modifiers
    pub category: FunctionCategory,
    /// Whether the function has type parameters (generics)
    pub has_type_parameters: bool,
    /// Number of parameters the function accepts
    pub parameter_count: usize,
}

impl FunctionTypeInfo {
    /// Check if this function can be called from transactions
    pub fn is_transaction_callable(&self) -> bool {
        self.is_entry
    }
    
    /// Check if this function is accessible from other modules
    pub fn is_externally_accessible(&self) -> bool {
        matches!(self.visibility, FunctionVisibility::Public | FunctionVisibility::PublicFriend)
    }
    
    /// Get a human-readable description of the function type
    pub fn description(&self) -> String {
        let mut desc = String::new();
        
        match self.visibility {
            FunctionVisibility::Public => desc.push_str("public"),
            FunctionVisibility::PublicFriend => desc.push_str("public(friend)"),
            FunctionVisibility::Private => desc.push_str("private"),
        }
        
        if self.is_entry {
            desc.push_str(" entry");
        }
        
        if self.is_native {
            desc.push_str(" native");
        }
        
        desc.push_str(" function");
        
        if self.has_type_parameters {
            desc.push_str(" (generic)");
        }
        
        desc
    }
}



impl FunctionAnalysis {
    /// Create a new FunctionAnalysis instance
    pub fn new(
        contract: String,
        function: String,
        source: String,
        location: LocationInfo,
        parameters: Vec<Parameter>,
        calls: Vec<FunctionCall>,
    ) -> Self {
        Self {
            contract,
            function,
            source,
            location,
            parameters,
            calls,
        }
    }

    /// Convert the analysis result to JSON string
    pub fn to_json(&self) -> AnalyzerResult<String> {
        serde_json::to_string_pretty(self).map_err(AnalyzerError::JsonError)
    }

    /// Create FunctionAnalysis from JSON string
    pub fn from_json(json: &str) -> AnalyzerResult<Self> {
        serde_json::from_str(json).map_err(AnalyzerError::JsonError)
    }
}

impl LocationInfo {
    /// Create a new LocationInfo instance
    pub fn new(file: PathBuf, start_line: u32, end_line: u32) -> Self {
        Self {
            file,
            start_line,
            end_line,
        }
    }

    /// Get the number of lines spanned by this location
    pub fn line_count(&self) -> u32 {
        if self.end_line >= self.start_line {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }
}

impl Parameter {
    /// Create a new Parameter instance
    pub fn new(name: String, type_: String) -> Self {
        Self { name, type_ }
    }
}

impl FunctionCall {
    /// Create a new FunctionCall instance
    pub fn new(file: PathBuf, function: String, module: String) -> Self {
        Self {
            file,
            function,
            module,
        }
    }
}

