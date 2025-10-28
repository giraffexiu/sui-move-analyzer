#!/usr/bin/env python3
"""
Setup script for move-function-analyzer Python package.
"""

from setuptools import setup, find_packages
from setuptools.command.install import install
import os
import shutil
import subprocess
import sys
from pathlib import Path

class CustomInstallCommand(install):
    """Custom installation command to build and install the Rust binary."""
    
    def run(self):
        # Build the Rust binary
        self.build_rust_binary()
        install.run(self)
    
    def build_rust_binary(self):
        """Build the Rust binary and copy it to the package directory."""
        print("Building Rust binary...")
        
        # Get the directory containing this setup.py
        setup_dir = Path(__file__).parent.absolute()
        rust_dir = setup_dir / "src" / "beta-2024"
        
        # Build the Rust project
        try:
            subprocess.run(
                ["cargo", "build", "--release"],
                cwd=rust_dir,
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            print(f"Failed to build Rust binary: {e}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            sys.exit(1)
        
        # Copy the binary to the package directory
        binary_name = "move-function-analyzer"
        if sys.platform == "win32":
            binary_name += ".exe"
        
        src_binary = rust_dir / "target" / "release" / binary_name
        dst_dir = setup_dir / "sui_move_analyzer" / "bin"
        dst_dir.mkdir(parents=True, exist_ok=True)
        dst_binary = dst_dir / binary_name
        
        if src_binary.exists():
            shutil.copy2(src_binary, dst_binary)
            # Make it executable on Unix systems
            if sys.platform != "win32":
                os.chmod(dst_binary, 0o755)
            print(f"Copied binary to {dst_binary}")
        else:
            print(f"Binary not found at {src_binary}")
            sys.exit(1)

# Read the README file
readme_path = Path(__file__).parent / "README.md"
long_description = ""
if readme_path.exists():
    with open(readme_path, "r", encoding="utf-8") as f:
        long_description = f.read()

setup(
    name="sui-move-analyzer",
    version="1.0.0",
    author="Move Contributors",
    author_email="opensource@movebit.xyz",
    description="A Python library for analyzing Move functions in Sui Move projects",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/movebit/sui-move-analyzer",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Software Development :: Compilers",
    ],
    python_requires=">=3.7",
    install_requires=[
        "typing-extensions>=3.7.4; python_version<'3.8'",
    ],
    cmdclass={
        'install': CustomInstallCommand,
    },
    include_package_data=True,
    package_data={
        "sui_move_analyzer": ["bin/*"],
    },
    keywords="move blockchain sui analysis static-analysis",
    project_urls={
        "Bug Reports": "https://github.com/movebit/sui-move-analyzer/issues",
        "Source": "https://github.com/movebit/sui-move-analyzer",
    },
)