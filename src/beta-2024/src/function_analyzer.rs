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

/// Represents the complete analysis result for a Move function
/// Contains all relevant information about function signature, location, and dependencies
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionAnalysis {
    pub contract: String,
    pub function: String,
    pub source: String,
    pub location: LocationInfo,
    #[serde(rename = "parameter")]
    pub parameters: Vec<Parameter>,
    pub calls: Vec<FunctionCall>,
}

/// Contains location information for a function in the source code
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocationInfo {
    pub file: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
}

/// Represents a function parameter with its name and type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

/// Represents a function call made within the analyzed function
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub file: PathBuf,
    pub function: String,
    pub module: String,
}

/// Error types that can occur during function analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("Invalid project path: {0}")]
    InvalidProjectPath(PathBuf),

    #[error("Move.toml file not found or invalid")]
    InvalidMoveToml,

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Type resolution error: {0}")]
    TypeResolutionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

/// Type alias for analyzer results
pub type AnalyzerResult<T> = Result<T, AnalyzerError>;

/// Utility struct for loading and validating Move projects
pub struct ProjectLoader;

impl ProjectLoader {
    /// Load and validate a Move project from the given path
    pub fn load_project(project_path: PathBuf) -> AnalyzerResult<Project> {
        Self::validate_move_project(&project_path)?;

        let _manifest = Self::parse_move_toml(&project_path)?;

        let mut multi_project = MultiProject::new();

        let implicit_deps = crate::implicit_deps();

        let errors = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let errors_clone = errors.clone();
        let error_reporter = move |error: String| {
            errors_clone.borrow_mut().push(error);
        };

        let project = Project::new(
            project_path.clone(),
            &mut multi_project,
            error_reporter,
            implicit_deps,
        ).map_err(|e| AnalyzerError::AnalysisError(format!("Failed to load project: {}", e)))?;

        let errors_vec = errors.borrow();
        if !errors_vec.is_empty() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Project loading errors: {}",
                errors_vec.join("; ")
            )));
        }

        if !project.load_ok() {
            log::warn!("Project did not load completely - some dependencies may be missing, but proceeding with available source code");
        }

        Ok(project)
    }

    /// Validate that the project path contains a valid Move project structure
    fn validate_move_project(project_path: &Path) -> AnalyzerResult<()> {
        if !project_path.exists() {
            return Err(AnalyzerError::InvalidProjectPath(project_path.to_path_buf()));
        }

        if !project_path.is_dir() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Project path is not a directory: {}",
                project_path.display()
            )));
        }

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

        Self::validate_directory_structure(project_path)?;

        Ok(())
    }

    /// Validate the directory structure of a Move project
    fn validate_directory_structure(project_path: &Path) -> AnalyzerResult<()> {
        let sources_path = project_path.join("sources");
        if sources_path.exists() {
            if !sources_path.is_dir() {
                return Err(AnalyzerError::AnalysisError(format!(
                    "sources path exists but is not a directory: {}",
                    sources_path.display()
                )));
            }

            match fs::read_dir(&sources_path) {
                Ok(_) => {},
                Err(e) => {
                    return Err(AnalyzerError::AnalysisError(format!(
                        "Cannot read sources directory: {}",
                        e
                    )));
                }
            }

            Self::validate_move_files_in_directory(&sources_path, "sources")?;
        }

        let tests_path = project_path.join("tests");
        if tests_path.exists() {
            if !tests_path.is_dir() {
                return Err(AnalyzerError::AnalysisError(format!(
                    "tests path exists but is not a directory: {}",
                    tests_path.display()
                )));
            }

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

        let scripts_path = project_path.join("scripts");
        if scripts_path.exists() {
            if !scripts_path.is_dir() {
                return Err(AnalyzerError::AnalysisError(format!(
                    "scripts path exists but is not a directory: {}",
                    scripts_path.display()
                )));
            }

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

    /// Validate Move files in a specific directory
    fn validate_move_files_in_directory(dir_path: &Path, dir_name: &str) -> AnalyzerResult<()> {
        let mut has_move_files = false;
        let mut validation_errors = Vec::new();

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

        if !validation_errors.is_empty() {
            return Err(AnalyzerError::AnalysisError(format!(
                "Validation errors in {} directory: {}",
                dir_name,
                validation_errors.join("; ")
            )));
        }

        if !has_move_files {
            log::debug!("No .move files found in {} directory: {}", dir_name, dir_path.display());
        }

        Ok(())
    }

    /// Parse and validate the Move.toml manifest file
    fn parse_move_toml(project_path: &Path) -> AnalyzerResult<SourceManifest> {
        let move_toml_path = project_path.join("Move.toml");

        let manifest = parse_move_manifest_from_file(project_path)
            .map_err(|e| {
                AnalyzerError::ParseError(format!(
                    "Failed to parse Move.toml at {}: {}",
                    move_toml_path.display(),
                    e
                ))
            })?;

        Self::validate_manifest_content(&manifest)?;

        Ok(manifest)
    }

    /// Validate the content of the parsed manifest
    fn validate_manifest_content(manifest: &SourceManifest) -> AnalyzerResult<()> {
        if manifest.package.name.as_str().is_empty() {
            return Err(AnalyzerError::ParseError(
                "Package name cannot be empty in Move.toml".to_string()
            ));
        }

        let package_name = manifest.package.name.as_str();
        if !package_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AnalyzerError::ParseError(format!(
                "Invalid package name '{}': must contain only alphanumeric characters, underscores, and hyphens",
                package_name
            )));
        }

        if let Some(edition) = &manifest.package.edition {
            let edition_str = match edition {
                &Edition::E2024_BETA => "2024.beta",
                &Edition::E2024_ALPHA => "2024.alpha",
                _ => "unknown",
            };
            log::debug!("Project uses edition: {}", edition_str);
        }

        if let Some(ref addresses) = manifest.addresses {
            for (name, address_opt) in addresses {
                if name.as_str().is_empty() {
                    return Err(AnalyzerError::ParseError(
                        "Address name cannot be empty".to_string()
                    ));
                }

                if let Some(address) = address_opt {
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

        for (dep_name, _dependency) in &manifest.dependencies {
            if dep_name.as_str().is_empty() {
                return Err(AnalyzerError::ParseError(
                    "Dependency name cannot be empty".to_string()
                ));
            }
        }

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

/// Represents a function definition with its module context
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub function: Function,
    pub module_info: ModuleInfo,
    pub location: Loc,
}

/// Module information including address and file location
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleInfo {
    pub address: AccountAddress,
    pub name: Symbol,
    pub file_path: PathBuf,
}

/// Type resolver for converting Move types to string representations
pub struct TypeResolver<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> TypeResolver<'a> {
    /// Create a new type resolver
    pub fn new(_project: &'a Project, _context: &'a ProjectContext) -> Self {
        Self { _phantom: std::marker::PhantomData }
    }

    /// Convert a Move type to its string representation
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

    fn format_unit_type(&self) -> String {
        "()".to_string()
    }

    fn format_apply_type(&self, name_access_chain: &move_compiler::parser::ast::NameAccessChain) -> String {
        self.name_access_chain_to_string(name_access_chain)
    }

    fn format_reference_type(&self, is_mut: bool, inner_type: &Type) -> String {
        let mut_str = if is_mut { "mut " } else { "" };
        format!("&{}{}", mut_str, self.type_to_string(inner_type))
    }

    fn format_function_type(&self, params: &[Type], return_type: &Type) -> String {
        let param_strings: Vec<String> = params.iter()
            .map(|t| self.type_to_string(t))
            .collect();
        format!("|{}| -> {}", param_strings.join(", "), self.type_to_string(return_type))
    }

    fn format_multiple_type(&self, types: &[Type]) -> String {
        let type_strings: Vec<String> = types.iter()
            .map(|t| self.type_to_string(t))
            .collect();
        format!("({})", type_strings.join(", "))
    }

    fn format_unresolved_error(&self) -> String {
        "UnresolvedError".to_string()
    }

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

    fn format_single_name_entry(&self, path_entry: &move_compiler::parser::ast::PathEntry) -> String {
        let name = path_entry.name.value.as_str();

        if self.is_basic_move_type(name) {
            return name.to_string();
        }

        if let Some(type_args) = &path_entry.tyargs {
            let type_arg_strings: Vec<String> = type_args.value.iter()
                .map(|t| self.type_to_string(t))
                .collect();
            format!("{}<{}>", name, type_arg_strings.join(", "))
        } else {
            name.to_string()
        }
    }

    fn format_path_name_access(&self, name_path: &move_compiler::parser::ast::NamePath) -> String {
        match &name_path.root.name.value {
            move_compiler::parser::ast::LeadingNameAccess_::Name(name) => {
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

    fn is_basic_move_type(&self, name: &str) -> bool {
        matches!(name,
            "u8" | "u16" | "u32" | "u64" | "u128" | "u256" |
            "bool" | "address" | "signer" | "vector"
        )
    }

    pub fn resolve_struct_type(&self, struct_name: &str, type_args: Option<&[Type]>) -> String {
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

    fn find_qualified_struct_name(&self, struct_name: &str) -> Option<String> {

        Some(struct_name.to_string())
    }

    pub fn format_vector_type(&self, element_type: &Type) -> String {
        format!("vector<{}>", self.type_to_string(element_type))
    }

    pub fn is_resource_type(&self, type_name: &str) -> bool {

        type_name.contains("Coin") ||
        type_name.contains("Object") ||
        type_name.contains("UID") ||
        type_name.ends_with("Cap") ||
        type_name.ends_with("Witness")
    }

    pub fn generate_readable_type_string(&self, type_: &Type) -> String {
        let base_string = self.type_to_string(type_);

        self.enhance_type_readability(&base_string)
    }

    fn enhance_type_readability(&self, type_string: &str) -> String {
        let enhanced = type_string
            .replace("<", " <")
            .replace(">", "> ")
            .replace("  ", " ")
            .trim()
            .to_string();

        enhanced.replace(" <", "<").replace("> ", ">")
    }

    pub fn handle_nested_generics(&self, base_type: &str, type_args: &[Type]) -> String {
        if type_args.is_empty() {
            return base_type.to_string();
        }

        let mut formatted_args = Vec::new();

        for type_arg in type_args {
            let arg_string = self.type_to_string(type_arg);

            let formatted_arg = if arg_string.contains('<') && arg_string.contains('>') {
                self.format_deeply_nested_type(&arg_string)
            } else {
                arg_string
            };

            formatted_args.push(formatted_arg);
        }

        format!("{}<{}>", base_type, formatted_args.join(", "))
    }
    fn format_deeply_nested_type(&self, type_string: &str) -> String {
        let depth = type_string.matches('<').count();

        if depth > 2 {
            self.simplify_deep_nesting(type_string)
        } else {
            type_string.to_string()
        }
    }

    fn simplify_deep_nesting(&self, type_string: &str) -> String {
        type_string.to_string()
    }

    pub fn resolve_resource_type_with_capabilities(&self, type_name: &str) -> (String, Vec<String>) {
        let capabilities = self.infer_type_capabilities(type_name);
        (type_name.to_string(), capabilities)
    }
    fn infer_type_capabilities(&self, type_name: &str) -> Vec<String> {
        let mut capabilities = Vec::new();

        if self.is_basic_move_type(type_name) {
            capabilities.extend_from_slice(&["copy".to_string(), "drop".to_string(), "store".to_string()]);
        }
        else if self.is_resource_type(type_name) {
            capabilities.push("key".to_string());
            if type_name.contains("Store") || type_name.ends_with("Data") {
                capabilities.push("store".to_string());
            }
        }
        else if type_name.starts_with("vector<") {
            capabilities.extend_from_slice(&["store".to_string()]);
        }
        else {
            capabilities.push("unknown".to_string());
        }

        capabilities
    }

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
    fn contains_generics(&self, type_string: &str) -> bool {
        type_string.contains('<') && type_string.contains('>')
    }
    fn calculate_type_complexity(&self, type_: &Type) -> u32 {
        match &type_.value {
            Type_::Unit => 0,
            Type_::Apply(name_access) => {
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
                2
            }
        }
    }
}

/// Comprehensive type information including metadata and capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub type_string: String,
    pub readable_string: String,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub is_generic: bool,
    pub complexity_level: u32,
    pub capabilities: Vec<String>,
}

/// Function visibility levels in Move
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionVisibility {
    Public,
    PublicFriend,
    Private,
}

/// Function categories based on visibility and special attributes
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCategory {
    Public,
    PublicFriend,
    Private,
    Entry,
    Native,
}

/// Detailed function type information and metadata
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTypeInfo {
    pub visibility: FunctionVisibility,
    pub is_entry: bool,
    pub is_native: bool,
    pub category: FunctionCategory,
    pub has_type_parameters: bool,
    pub parameter_count: usize,
}

impl FunctionTypeInfo {
    /// Check if function can be called in transactions
    pub fn is_transaction_callable(&self) -> bool {
        self.is_entry
    }

    /// Check if function is accessible from outside the module
    pub fn is_externally_accessible(&self) -> bool {
        matches!(self.visibility, FunctionVisibility::Public | FunctionVisibility::PublicFriend)
    }

    /// Generate a human-readable description of the function
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
    /// Create a new function analysis instance
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

    /// Convert analysis to JSON string
    pub fn to_json(&self) -> AnalyzerResult<String> {
        serde_json::to_string_pretty(self).map_err(AnalyzerError::JsonError)
    }

    /// Create analysis from JSON string
    pub fn from_json(json: &str) -> AnalyzerResult<Self> {
        serde_json::from_str(json).map_err(AnalyzerError::JsonError)
    }
}

impl LocationInfo {
    pub fn new(file: PathBuf, start_line: u32, end_line: u32) -> Self {
        Self {
            file,
            start_line,
            end_line,
        }
    }

    /// Calculate the number of lines in the location range
    pub fn line_count(&self) -> u32 {
        if self.end_line >= self.start_line {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }
}

impl Parameter {
    pub fn new(name: String, type_: String) -> Self {
        Self { name, type_ }
    }
}

impl FunctionCall {
    pub fn new(file: PathBuf, function: String, module: String) -> Self {
        Self {
            file,
            function,
            module,
        }
    }
}