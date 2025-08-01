# CppSage

CppSage is a command-line tool for managing C++ projects. It simplifies the process of creating, building, and running C++ applications.

## Features

- **Project Scaffolding**: Quickly create a new C++ project with a standard directory structure.
- **Dependency Management**: Easily manage project dependencies using Conan.
- **Build System**: Uses CMake for building the project.
- **Cross-Platform**: Designed to work on different operating systems.

## Installation

1. **Install Rust**: If you don't have Rust installed, you can get it from [rust-lang.org](https://www.rust-lang.org/).
2. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/cppsage.git
   cd cppsage
   ```
3. **Build the project**:
   ```bash
   cargo build --release
   ```
4. **Add to PATH**: Add the `target/release` directory to your system's PATH to run `cppsage` from anywhere.

## Usage

### Create a new project

```bash
cppsage new <project-name>
```

This will create a new directory with the specified project name and set up a basic C++ project structure.

### Install dependencies

```bash
cppsage install
```

This command reads the `packages/requirements.txt` file, installs the specified dependencies using Conan, and updates the `CMakeLists.txt` file.

### Compile the project

```bash
cppsage compile
```

This will compile the project using CMake and Ninja. The build artifacts will be placed in the `build` directory.

### Run the project

```bash
cppsage run
```

This command first compiles the project and then runs the executable.

### Check for required tools

```bash
cppsage doctor
```

This command checks if all the required tools (CMake, Ninja, Conan, etc.) are installed and available in the PATH.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.
