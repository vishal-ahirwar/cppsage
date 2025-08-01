use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new C++ project
    New {
        /// The name of the project
        #[arg(required = true)]
        name: String,
    },
    /// Install dependencies
    Install,
    /// Compile the project
    Compile,
    /// Compile and run the project
    Run,
    /// Debug the project
    Debug,
    /// Check for required tools
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            println!("{} {} '{}'", "Creating new project:".green(), "sage".bold(), name.bold());
            if let Err(e) = create_project(name) {
                eprintln!("{} {}", "Error:".red(), e);
            } else {
                println!("{} Project '{}' created successfully!", "Success:".green(), name);
            }
        }
        Commands::Install => {
            if let Err(e) = install_dependencies() {
                eprintln!("{} {}", "Error:".red(), e);
            }
        }
        Commands::Compile => {
            if let Err(e) = compile_project() {
                eprintln!("{} {}", "Error:".red(), e);
            }
        }
        Commands::Run => {
            if let Err(e) = run_project() {
                eprintln!("{} {}", "Error:".red(), e);
            }
        }
        Commands::Debug => {
            println!("{}", "Debugging project...".green());
            // Actual implementation will go here
        }
        Commands::Doctor => {
            println!("{}", "Checking for required tools...".green());
            check_tools();
        }
    }
}

fn compile_project() -> Result<(), std::io::Error> {
    println!("{}", "Configuring project with CMake...".green());

    let build_dir = "build";
    fs::create_dir_all(build_dir)?;
    
    let toolchain_path = "packages/install/conan_toolchain.cmake";

    // Configure with CMake
    let configure_output = Command::new("cmake")
        .args(&[
            "-S", ".",
            "-B", build_dir,
            "-G", "Ninja",
            &format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain_path)
        ])
        .output()?;

    if !configure_output.status.success() {
        let stderr = String::from_utf8_lossy(&configure_output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("CMake configuration failed:\n{}", stderr)));
    }
    println!("{}", String::from_utf8_lossy(&configure_output.stdout));
    println!("{}", String::from_utf8_lossy(&configure_output.stderr));


    println!("{}", "Compiling project with CMake...".green());
    // Build with CMake
    let build_output = Command::new("cmake")
        .args(&["--build", build_dir])
        .output()?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("CMake build failed:\n{}", stderr)));
    }
    println!("{}", String::from_utf8_lossy(&build_output.stdout));
     println!("{}", String::from_utf8_lossy(&build_output.stderr));
    
    println!("{} Project compiled successfully!", "Success:".green());

    Ok(())
}

fn run_project() -> Result<(), std::io::Error> {
    // First, compile the project
    compile_project()?;

    println!("{}", "Running project...".green());

    let project_name = env::current_dir()?.file_name().unwrap().to_str().unwrap().to_string();
    
    let exe_path = if cfg!(target_os = "windows") {
        Path::new("build").join(&project_name).join(format!("{}.exe", project_name))
    } else {
        Path::new("build").join(&project_name).join(&project_name)
    };

    if !exe_path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Executable not found at: {:?}", exe_path)));
    }

    let run_output = Command::new(exe_path).output()?;

    println!("--- Program Output ---");
    println!("{}", String::from_utf8_lossy(&run_output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&run_output.stderr));
    println!("--- End Program Output ---");

    if !run_output.status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Project execution failed."));
    }

    Ok(())
}


fn install_dependencies() -> Result<(), std::io::Error> {
    println!("{}", "Installing dependencies...".green());

    // 1. Parse requirements.txt
    let requirements_path = Path::new("packages/requirements.txt");
    if !requirements_path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "packages/requirements.txt not found. Are you in the project root?"));
    }
    let file = fs::File::open(requirements_path)?;
    let reader = BufReader::new(file);
    let dependencies: Vec<String> = reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    if dependencies.is_empty() {
        println!("{}", "No dependencies to install.".yellow());
        return Ok(());
    }
    
    println!("Found dependencies: {:?}", dependencies);

    // 2. Create conanfile.txt
    let conanfile_path = Path::new("conanfile.txt");
    let mut conanfile_content = "[requires]\n".to_string();
    for dep in &dependencies {
        conanfile_content.push_str(dep);
        conanfile_content.push('\n');
    }
    conanfile_content.push_str("\n[generators]\n");
    conanfile_content.push_str("CMakeDeps\n");
    conanfile_content.push_str("CMakeToolchain\n");
    fs::write(conanfile_path, conanfile_content)?;

    // 3. Run conan install
    println!("{}", "Running conan install...".green());
    let output = Command::new("conan")
        .args(&["install", ".", "--build=missing", "--output-folder=packages/install"])
        .output()?;

    // 4. Delete conanfile.txt
    fs::remove_file(conanfile_path)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Conan install failed:\n{}", stderr)));
    }
    println!("{}", String::from_utf8_lossy(&output.stdout));


    // 5. Update CMakeLists.txt
    println!("{}", "Updating CMakeLists.txt...".green());
    let project_name = env::current_dir()?.file_name().unwrap().to_str().unwrap().to_string();
    let cmake_path = Path::new(&project_name).join("CMakeLists.txt");
    
    let mut cmake_content = fs::read_to_string(&cmake_path)?;

    let mut new_deps = String::new();
    for dep in dependencies {
        let dep_name = dep.split('/').next().unwrap();
        new_deps.push_str(&format!("find_package({})\n", dep_name));
        new_deps.push_str(&format!("target_link_libraries({} PRIVATE {}::{})\n", project_name, dep_name, dep_name));
    }

    let start_marker = "# cppsage:dependencies_start";
    let end_marker = "# cppsage:dependencies_end";

    if let (Some(start), Some(end)) = (cmake_content.find(start_marker), cmake_content.find(end_marker)) {
        let range = start + start_marker.len()..end;
        cmake_content.replace_range(range, &format!("\n{}\n", new_deps));
        fs::write(&cmake_path, cmake_content)?;
        println!("{} Successfully updated CMakeLists.txt", "Success:".green());
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not find dependency markers in CMakeLists.txt"));
    }

    Ok(())
}


fn create_project(project_name: &str) -> Result<(), std::io::Error> {
    let root = Path::new(project_name);
    if root.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("Directory '{}' already exists.", project_name)));
    }

    // Create directory structure
    fs::create_dir_all(root.join("build/windows"))?;
    fs::create_dir_all(root.join("cmake"))?;
    fs::create_dir_all(root.join(project_name).join("include"))?;
    fs::create_dir_all(root.join(project_name).join("src"))?;
    fs::create_dir_all(root.join("install"))?;
    fs::create_dir_all(root.join("packages"))?;
    fs::create_dir_all(root.join("res"))?;

    // Create files
    fs::write(root.join(".clang-format"), CLANG_FORMAT_CONTENT)?;
    fs::write(root.join(".clang-tidy"), "")?; // Empty file
    fs::write(root.join(".clangd"), CLANGD_CONTENT)?;
    fs::write(root.join(".editorconfig"), EDITORCONFIG_CONTENT)?;
    fs::write(root.join(".gitignore"), GITIGNORE_CONTENT)?;
    fs::write(root.join("CMakeLists.txt"), &cmake_lists_top(project_name))?;
    fs::write(root.join("cmake/config.cmake"), CONFIG_CMAKE_CONTENT)?;
    fs::write(root.join(project_name).join("CMakeLists.txt"), &cmake_lists_sub(project_name))?;
    fs::write(root.join(project_name).join("src").join("main.cpp"), MAIN_CPP_CONTENT)?;
    fs::write(root.join("packages/requirements.txt"), REQUIREMENTS_TXT_CONTENT)?;

    Ok(())
}

fn check_tools() {
    println!("\n{}", "cppsage doctor".bold().underline());
    check_tool("cmake", &["--version"], "winget install Kitware.CMake");
    check_tool("ninja", &["--version"], "winget install Kitware.Ninja");
    check_tool("conan", &["--version"], "pip install conan");
    check_tool("clang", &["--version"], "winget install LLVM.LLVM");

    if cfg!(target_os = "windows") {
        check_vs_build_tools();
    }
}

fn check_tool(tool: &str, args: &[&str], install_hint: &str) {
    print!("- {}: ", tool.bold());
    match Command::new(tool).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string();
                println!("{} {}", "OK".green(), version.dimmed());
            } else {
                println!("{}", "Not found".red());
                println!("  {}", install_hint.cyan());
            }
        }
        Err(_) => {
            println!("{}", "Not found".red());
            println!("  {}", install_hint.cyan());
        }
    }
}

#[cfg(target_os = "windows")]
fn check_vs_build_tools() {
    print!("- {}: ", "Visual Studio Build Tools".bold());
    
    let program_files = env::var("ProgramFiles(x86)").unwrap_or_else(|_|"C:\\Program Files (x86)".to_string());
    let vswhere_path = Path::new(&program_files).join("Microsoft Visual Studio/Installer/vswhere.exe");

    if !vswhere_path.exists() {
        println!("{}", "Not found".red());
        println!("  (vswhere.exe not found at expected path)");
        println!("  {}", "Install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/".cyan());
        return;
    }

    let result = Command::new(vswhere_path)
        .args(&["-latest", "-property", "displayName"])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                 let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("{} {}", "OK".green(), version.dimmed());
            } else {
                println!("{}", "Not found".red());
                println!("  {}", "Install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/".cyan());
            }
        }
        Err(_) => {
            println!("{}", "Not found".red());
            println!("  {}", "Install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/".cyan());
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn check_vs_build_tools() {
    // Do nothing on non-Windows platforms
}


// Boilerplate content
const CLANG_FORMAT_CONTENT: &str = r#"
Language: Cpp
BasedOnStyle: LLVM
AccessModifierOffset: -2
AlignAfterOpenBracket: Align
AlignConsecutiveAssignments: false
AlignConsecutiveDeclarations: false
AlignEscapedNewlines: Left
AlignOperands: Align
AlignTrailingComments: true
AllowAllParametersOfDeclarationOnNextLine: true
AllowShortBlocksOnASingleLine: false
AllowShortCaseLabelsOnASingleLine: false
AllowShortFunctionsOnASingleLine: All
AllowShortIfStatementsOnASingleLine: Never
AllowShortLoopsOnASingleLine: false
AlwaysBreakAfterDefinitionReturnType: None
AlwaysBreakAfterReturnType: None
AlwaysBreakBeforeMultilineStrings: false
AlwaysBreakTemplateDeclarations: Yes
BinPackArguments: true
BinPackParameters: true
BraceWrapping:
  AfterClass: false
  AfterControlStatement: false
  AfterEnum: false
  AfterFunction: false
  AfterNamespace: false
  AfterObjCDeclaration: false
  AfterStruct: false
  AfterUnion: false
  AfterExternBlock: false
  BeforeCatch: false
  BeforeElse: false
  IndentBraces: false
BreakBeforeBraces: Custom
BreakBeforeBinaryOperators: None
BreakBeforeInheritanceComma: false
BreakBeforeTernaryOperators: true
BreakConstructorInitializers: BeforeColon
ColumnLimit: 80
ConstructorInitializerAllOnOneLineOrOnePerLine: false
ConstructorInitializerIndentWidth: 4
ContinuationIndentWidth: 4
Cpp11BracedListStyle: true
DerivePointerAlignment: false
DisableFormat: false
ExperimentalAutoDetectBinPacking: false
FixNamespaceComments: true
ForEachMacros:
  - foreach
  - Q_FOREACH
  - BOOST_FOREACH
IncludeBlocks: Preserve
IncludeCategories:
  - Regex: '^"(<project_name>|config)\.h"'
    Priority: 1
  - Regex: '^<.*\.h>'
    Priority: 2
  - Regex: '^<.*'
    Priority: 3
  - Regex: '.*'
    Priority: 4
IncludeIsMainRegex: '(Test)?\.cpp$'
IndentCaseLabels: false
IndentPPDirectives: None
IndentWidth: 4
IndentWrappedFunctionNames: false
JavaScriptQuotes: Leave
JavaScriptWrapImports: true
KeepEmptyLinesAtTheStartOfBlocks: true
MacroBlockBegin: ''
MacroBlockEnd: ''
MaxEmptyLinesToKeep: 1
NamespaceIndentation: None
ObjCBlockIndentWidth: 4
ObjCSpaceAfterProperty: false
ObjCSpaceBeforeProtocolList: true
PenaltyBreakAssignment: 2
PenaltyBreakBeforeFirstCallParameter: 19
PenaltyBreakComment: 300
PenaltyBreakFirstLessLess: 120
PenaltyBreakString: 1000
PenaltyReturnTypeOnItsOwnLine: 60
PointerAlignment: Left
ReflowComments: true
SortIncludes: true
SortUsingDeclarations: true
SpaceAfterCStyleCast: false
SpaceAfterTemplateKeyword: true
SpaceBeforeAssignmentOperators: true
SpaceBeforeCpp11BracedList: false
SpaceBeforeCtorInitializerColon: true
SpaceBeforeInheritanceColon: true
SpaceBeforeParens: ControlStatements
SpaceInEmptyParentheses: false
SpacesBeforeTrailingComments: 2
SpacesInAngles: false
SpacesInContainerLiterals: false
SpacesInCStyleCastParentheses: false
SpacesInParentheses: false
Standard: Cpp11
TabWidth: 4
UseTab: Never
"#;

const CLANGD_CONTENT: &str = r#"
CompileFlags:
  Add: [-std=c++17]
"#;

const EDITORCONFIG_CONTENT: &str = r#"
root = true

[*]
indent_style = space
indent_size = 4
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true
"#;

const GITIGNORE_CONTENT: &str = r#"
# CMake
build/
install/
*.VC.db
*.VC.VC.opendb

# Visual Studio
.vs/
*.suo
*.user
*.sln.docstates

# Packages
packages/

# Misc
*.log
"#;

fn cmake_lists_top(project_name: &str) -> String {
    format!(r#"
cmake_minimum_required(VERSION 3.15)

# Conan package management
include(cmake/config.cmake)

project({} VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_subdirectory({})
"#, project_name, project_name)
}

const CONFIG_CMAKE_CONTENT: &str = r#"
# This file is managed by cppsage.
# Manual edits might be overwritten.

# Check if conan_toolchain.cmake exists
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/packages/install/conan_toolchain.cmake")
    include("${CMAKE_CURRENT_SOURCE_DIR}/packages/install/conan_toolchain.cmake")
else()
    message(WARNING "Conan toolchain not found. Run 'sage install' to generate it.")
endif()
"#;

fn cmake_lists_sub(project_name: &str) -> String {
    format!(r#"
add_executable({0}
    src/main.cpp
)

target_include_directories({0} PUBLIC
    "${{CMAKE_CURRENT_SOURCE_DIR}}/include"
)

# cppsage:dependencies_start
# cppsage:dependencies_end
"#, project_name)
}

const MAIN_CPP_CONTENT: &str = r#"
#include <iostream>

int main() {
    std::cout << "Hello, world!" << std::endl;
    return 0;
}
"#;

const REQUIREMENTS_TXT_CONTENT: &str = r#"
# Add your dependencies here
# e.g. fmt/10.2.1
"#;