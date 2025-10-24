#!/usr/bin/env python3
"""
Installation script for the Move Function Analyzer Python package.
This script builds the Rust binary and installs the Python package.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path


def run_command(cmd, cwd=None, check=True):
    """Run a command and handle errors."""
    print(f"Running: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True)
        if result.stdout:
            print(result.stdout)
        return result
    except subprocess.CalledProcessError as e:
        print(f"Command failed with exit code {e.returncode}")
        if e.stdout:
            print(f"stdout: {e.stdout}")
        if e.stderr:
            print(f"stderr: {e.stderr}")
        if check:
            sys.exit(1)
        return e


def main():
    """Main installation process."""
    print("🚀 Installing Move Function Analyzer Python Package")
    print("=" * 50)
    
    # Get paths
    script_dir = Path(__file__).parent.absolute()
    rust_dir = script_dir.parent / "src" / "beta-2024"
    
    # Check if Rust project exists
    if not rust_dir.exists():
        print(f"❌ Rust project not found at {rust_dir}")
        print("Please make sure you're running this from the correct directory.")
        sys.exit(1)
    
    # Check if cargo is available
    try:
        subprocess.run(["cargo", "--version"], check=True, capture_output=True)
        print("✅ Cargo found")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ Cargo not found. Please install Rust and Cargo first.")
        print("Visit: https://rustup.rs/")
        sys.exit(1)
    
    # Build the Rust binary
    print("\n📦 Building Rust binary...")
    run_command(["cargo", "build", "--release"], cwd=rust_dir)
    
    # Check if binary was built
    binary_name = "move-function-analyzer"
    if sys.platform == "win32":
        binary_name += ".exe"
    
    binary_path = rust_dir / "target" / "release" / binary_name
    if not binary_path.exists():
        print(f"❌ Binary not found at {binary_path}")
        sys.exit(1)
    
    print(f"✅ Binary built successfully at {binary_path}")
    
    # Copy binary to package directory
    print("\n📋 Copying binary to package...")
    package_bin_dir = script_dir / "move_function_analyzer" / "bin"
    package_bin_dir.mkdir(parents=True, exist_ok=True)
    
    dst_binary = package_bin_dir / binary_name
    shutil.copy2(binary_path, dst_binary)
    
    # Make executable on Unix systems
    if sys.platform != "win32":
        os.chmod(dst_binary, 0o755)
    
    print(f"✅ Binary copied to {dst_binary}")
    
    # Install the Python package
    print("\n🐍 Installing Python package...")
    
    # First, try to install in development mode
    install_cmd = [sys.executable, "-m", "pip", "install", "-e", "."]
    result = run_command(install_cmd, cwd=script_dir, check=False)
    
    if result.returncode != 0:
        print("Development install failed, trying regular install...")
        install_cmd = [sys.executable, "-m", "pip", "install", "."]
        run_command(install_cmd, cwd=script_dir)
    
    print("✅ Python package installed successfully!")
    
    # Test the installation
    print("\n🧪 Testing installation...")
    try:
        import move_function_analyzer
        print(f"✅ Package imported successfully (version {move_function_analyzer.__version__})")
        
        # Test analyzer creation
        analyzer = move_function_analyzer.MoveFunctionAnalyzer()
        print("✅ Analyzer created successfully")
        
    except Exception as e:
        print(f"❌ Installation test failed: {e}")
        sys.exit(1)
    
    print("\n🎉 Installation completed successfully!")
    print("\nYou can now use the library:")
    print("```python")
    print("from move_function_analyzer import MoveFunctionAnalyzer")
    print("analyzer = MoveFunctionAnalyzer()")
    print("results = analyzer.analyze('/path/to/project', 'function_name')")
    print("```")
    
    print("\nOr run the examples:")
    print(f"python {script_dir / 'examples' / 'basic_usage.py'}")


if __name__ == "__main__":
    main()