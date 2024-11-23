#!/bin/bash

# setup_opencog_hyperon.sh
# Automated script to install and set up OpenCog Hyperon
# This script excludes the 'doc' repository as it is a protected special function.

set -euo pipefail  # Enable strict error handling

# -----------------------------------
# Function Definitions
# -----------------------------------

# Function to print informational messages
echo_info() {
    echo -e "\e[34m[INFO]\e[0m $1"
}

# Function to print error messages
echo_error() {
    echo -e "\e[31m[ERROR]\e[0m $1" >&2
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install system dependencies
install_system_dependencies() {
    echo_info "Updating package lists..."
    sudo apt-get update

    echo_info "Installing system dependencies..."
    sudo apt-get install -y \
        python3 \
        python3-dev \
        python3-pip \
        build-essential \
        cmake \
        libssl-dev \
        zlib1g-dev \
        libgtk-3-dev \
        curl \
        git \
        wget \
        librocksdb-dev \
        libgoogle-perftools-dev

    echo_info "System dependencies installed successfully."
}

# Function to install Rust
install_rust() {
    if command_exists rustc; then
        echo_info "Rust is already installed. Skipping Rust installation."
    else
        echo_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Load Rust environment
        source "$HOME/.cargo/env"
        echo_info "Rust installed successfully."
    fi
}

# Function to install cbindgen
install_cbindgen() {
    if command_exists cbindgen; then
        echo_info "cbindgen is already installed. Skipping installation."
    else
        echo_info "Installing cbindgen..."
        cargo install --force cbindgen
        echo_info "cbindgen installed successfully."
    fi
}

# Function to install Conan
install_conan() {
    if command_exists conan; then
        echo_info "Conan is already installed. Skipping installation."
    else
        echo_info "Installing Conan..."
        python3 -m pip install conan==2.5.0
        conan profile detect --force
        echo_info "Conan installed and profile set up successfully."
    fi
}

# Function to upgrade pip
upgrade_pip() {
    echo_info "Upgrading pip to version 23.1.2..."
    python3 -m pip install --upgrade pip==23.1.2
    echo_info "pip upgraded successfully."
}

# Function to install Google Benchmark from source
install_google_benchmark() {
    if [ -d "google-benchmark" ]; then
        echo_info "Google Benchmark already installed."
    else
        echo_info "Installing Google Benchmark from source..."
        git clone https://github.com/google/benchmark.git
        cd benchmark
        git checkout v1.5.2  # Specify the desired version
        mkdir -p build && cd build
        cmake -DCMAKE_BUILD_TYPE=Release ..
        make -j$(nproc)
        sudo make install
        cd ../..
        rm -rf benchmark
        echo_info "Google Benchmark installed successfully."
    fi
}

# Function to install RocksDB
install_rocksdb() {
    echo_info "Installing RocksDB..."
    sudo apt-get install -y librocksdb-dev
    echo_info "RocksDB installed successfully."
}

# Function to install GTK3
install_gtk3() {
    echo_info "Installing GTK3..."
    sudo apt-get install -y libgtk-3-dev
    echo_info "GTK3 installed successfully."
}

# Function to install MeTTa via PyPi
install_metta_pypi() {
    echo_info "Installing MeTTa interpreter from PyPi..."
    python3 -m pip install hyperon
    echo_info "MeTTa interpreter installed successfully."
}

# Function to install MeTTa via Docker
install_metta_docker() {
    echo_info "Pulling and running MeTTa Docker image..."
    docker run -ti trueagi/hyperon:latest
    echo_info "MeTTa Docker container running."
}

# Function to clone OpenCog Hyperon repository, excluding 'doc'
clone_opencog_hyperon() {
    REPO_URL="https://github.com/trueagi-io/hyperon-experimental.git"
    REPO_DIR="opencog-hyperon"

    if [ -d "$REPO_DIR" ]; then
        echo_info "opencog-hyperon repository already exists. Updating..."
        cd "$REPO_DIR"
        git pull
        git submodule update --init --recursive

        # Exclude 'doc' submodule if it exists
        if git config -f .gitmodules --get-regexp 'submodule\.doc\.path' > /dev/null 2>&1; then
            echo_info "Removing 'doc' submodule as it is a protected special function..."
            git submodule deinit -f doc
            git rm -f doc
            rm -rf .git/modules/doc
            # Remove the submodule entry from .gitmodules
            sed -i '/\[submodule "doc"\]/,/path = doc/d' .gitmodules
            echo_info "'doc' submodule removed successfully."
        else
            echo_info "'doc' submodule does not exist. No action needed."
        fi

        cd ..
    else
        echo_info "Cloning OpenCog Hyperon repository..."
        git clone "$REPO_URL" "$REPO_DIR"
        cd "$REPO_DIR"
        git submodule update --init --recursive

        # Exclude 'doc' submodule if it exists
        if git config -f .gitmodules --get-regexp 'submodule\.doc\.path' > /dev/null 2>&1; then
            echo_info "Removing 'doc' submodule as it is a protected special function..."
            git submodule deinit -f doc
            git rm -f doc
            rm -rf .git/modules/doc
            # Remove the submodule entry from .gitmodules
            sed -i '/\[submodule "doc"\]/,/path = doc/d' .gitmodules
            echo_info "'doc' submodule removed successfully."
        else
            echo_info "'doc' submodule does not exist. No action needed."
        fi

        cd ..
        echo_info "opencog-hyperon repository cloned successfully."
    fi
}

# Function to build Docker image from local repository
build_docker_image_local() {
    REPO_DIR="opencog-hyperon"
    IMAGE_NAME="trueagi/hyperon"

    echo_info "Building Docker image from local repository..."
    cd "$REPO_DIR"
    docker build -t "$IMAGE_NAME" .
    cd ..
    echo_info "Docker image '$IMAGE_NAME' built successfully."
}

# Function to build Docker image without local repository
build_docker_image_remote() {
    IMAGE_NAME="trueagi/hyperon"
    REMOTE_REPO_URL="https://github.com/trueagi-io/hyperon-experimental.git#main"

    echo_info "Building Docker image without local repository..."
    docker build \
        --build-arg BUILDKIT_CONTEXT_KEEP_GIT_DIR=1 \
        -t "$IMAGE_NAME" \
        "$REMOTE_REPO_URL"
    echo_info "Docker image '$IMAGE_NAME' built successfully."
}

# Function to build Docker image with build target
build_docker_image_build_target() {
    REPO_DIR="opencog-hyperon"
    IMAGE_NAME="trueagi/hyperon"

    echo_info "Building Docker image with build target..."
    cd "$REPO_DIR"
    docker build --target build -t "$IMAGE_NAME" .
    cd ..
    echo_info "Docker image '$IMAGE_NAME' built successfully with build target."
}

# Function to set up build using CMake
setup_build() {
    REPO_DIR="opencog-hyperon"
    BUILD_DIR="$REPO_DIR/build"

    echo_info "Setting up build directory with CMake..."
    mkdir -p "$BUILD_DIR"
    cd "$REPO_DIR/build"
    cmake ..
    cd ../..
    echo_info "CMake setup completed."
}

# Function to build and run tests
build_and_run_tests() {
    REPO_DIR="opencog-hyperon"

    echo_info "Building and running tests..."
    cd "$REPO_DIR"
    make
    make check
    cd ..
    echo_info "Build and tests completed successfully."
}

# Function to install Python module for development
install_python_module() {
    REPO_DIR="opencog-hyperon/python"

    echo_info "Installing Python module for development..."
    cd "$REPO_DIR"
    python3 -m pip install -e ./[dev]
    cd ../..
    echo_info "Python module installed successfully."
}

# Function to run Python unit tests
run_python_unit_tests() {
    REPO_DIR="opencog-hyperon"

    echo_info "Running Python unit tests..."
    cd "$REPO_DIR"
    pytest ./tests
    cd ..
    echo_info "Python unit tests completed successfully."
}

# Function to run MeTTa script
run_metta_script() {
    SCRIPT_NAME="<name>.metta"  # Replace <name> with actual script name
    echo_info "Running MeTTa script: $SCRIPT_NAME..."
    metta-py "./opencog-hyperon/tests/scripts/$SCRIPT_NAME"
    echo_info "MeTTa script executed successfully."
}

# Function to run REPL with Python support
run_repl_python() {
    REPO_DIR="opencog-hyperon"

    echo_info "Running REPL with Python support..."
    cd "$REPO_DIR"
    cargo run --features python --bin metta-repl
    cd ..
    echo_info "REPL running successfully."
}

# -----------------------------------
# Main Script Execution
# -----------------------------------

main() {
    # Step 1: Install system dependencies
    install_system_dependencies

    # Step 2: Install Rust
    install_rust

    # Load Rust environment
    source "$HOME/.cargo/env"

    # Step 3: Install cbindgen
    install_cbindgen

    # Step 4: Install Conan
    install_conan

    # Step 5: Upgrade pip
    upgrade_pip

    # Step 6: Install Google Benchmark
    install_google_benchmark

    # Step 7: Install RocksDB
    install_rocksdb

    # Step 8: Install GTK3
    install_gtk3

    # Step 9: Install MeTTa via PyPi
    install_metta_pypi

    # Alternatively, install MeTTa via Docker (uncomment if needed)
    # install_metta_docker

    # Step 10: Clone OpenCog Hyperon repository, excluding 'doc'
    clone_opencog_hyperon

    # Step 11: Build Docker image from local repository
    build_docker_image_local

    # Alternatively, build Docker image without local repository (uncomment if needed)
    # build_docker_image_remote

    # Alternatively, build Docker image with build target (uncomment if needed)
    # build_docker_image_build_target

    # Step 12: Set up build using CMake
    setup_build

    # Step 13: Build and run tests
    build_and_run_tests

    # Step 14: Install Python module for development
    install_python_module

    # Step 15: Run Python unit tests
    run_python_unit_tests

    # Step 16: Run MeTTa script (replace <name> with actual script name)
    # run_metta_script

    # Step 17: Run REPL with Python support
    # run_repl_python

    echo_info "OpenCog Hyperon setup and installation completed successfully."
}

# Execute the main function
main
