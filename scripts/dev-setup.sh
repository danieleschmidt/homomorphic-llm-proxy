#!/bin/bash
# Development environment setup script for FHE LLM Proxy
# Installs all necessary tools and dependencies for development

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check system requirements
check_system() {
    info "Checking system requirements..."
    
    # Check OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        info "Detected Linux system"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        info "Detected macOS system"
    else
        error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
    
    # Check CPU features
    if command_exists lscpu; then
        if lscpu | grep -q "aes"; then
            success "AES-NI instruction support detected"
        else
            warning "AES-NI instructions not detected - performance may be reduced"
        fi
        
        if lscpu | grep -q "avx2"; then
            success "AVX2 instruction support detected"
        else
            warning "AVX2 instructions not detected - performance may be reduced"
        fi
    fi
    
    # Check NVIDIA GPU
    if command_exists nvidia-smi; then
        info "NVIDIA GPU detected:"
        nvidia-smi --query-gpu=name,memory.total --format=csv,noheader
        success "GPU support available"
    else
        warning "NVIDIA GPU not detected - GPU features will be disabled"
    fi
}

# Install Rust and Cargo tools
install_rust_tools() {
    info "Installing Rust development tools..."
    
    # Check if Rust is installed
    if ! command_exists rustc; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        info "Rust already installed: $(rustc --version)"
    fi
    
    # Update Rust
    info "Updating Rust toolchain..."
    rustup update
    
    # Install components
    info "Installing Rust components..."
    rustup component add rustfmt clippy
    
    # Install cargo tools
    info "Installing Cargo development tools..."
    local tools=(
        "cargo-nextest"      # Better test runner
        "cargo-audit"        # Security audit
        "cargo-deny"         # Dependency checker
        "cargo-tarpaulin"    # Code coverage
        "cargo-flamegraph"   # Performance profiling
        "cargo-mutants"      # Mutation testing
        "cargo-bloat"        # Binary size analysis
        "cargo-outdated"     # Dependency updates
        "cargo-watch"        # File watching
        "cargo-edit"         # Cargo.toml editing
    )
    
    for tool in "${tools[@]}"; do
        if ! command_exists "$tool"; then
            info "Installing $tool..."
            cargo install "$tool"
        else
            info "$tool already installed"
        fi
    done
    
    success "Rust tools installation complete"
}

# Install Python development tools
install_python_tools() {
    info "Setting up Python development environment..."
    
    # Check Python version
    if ! command_exists python3; then
        error "Python 3 is not installed. Please install Python 3.9 or later."
        exit 1
    fi
    
    local python_version
    python_version=$(python3 --version | cut -d' ' -f2)
    info "Detected Python version: $python_version"
    
    # Create virtual environment
    if [[ ! -d "python/.venv" ]]; then
        info "Creating Python virtual environment..."
        cd python
        python3 -m venv .venv
        cd ..
    fi
    
    # Activate virtual environment and install dependencies
    info "Installing Python dependencies..."
    cd python
    source .venv/bin/activate
    
    # Upgrade pip
    pip install --upgrade pip
    
    # Install package in development mode
    pip install -e ".[dev]"
    
    # Install additional development tools
    pip install --upgrade \
        black \
        ruff \
        mypy \
        pytest \
        pytest-cov \
        pytest-asyncio \
        bandit \
        safety \
        pre-commit
    
    cd ..
    success "Python development environment setup complete"
}

# Install system dependencies
install_system_deps() {
    info "Installing system dependencies..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Detect Linux distribution
        if command_exists apt-get; then
            # Debian/Ubuntu
            info "Detected Debian/Ubuntu system"
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                pkg-config \
                libssl-dev \
                libcuda-dev \
                cuda-toolkit \
                gnuplot \
                valgrind \
                perf \
                strace \
                htop \
                jq \
                curl \
                git
                
        elif command_exists yum; then
            # RHEL/CentOS/Fedora
            info "Detected RHEL/CentOS/Fedora system"
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y \
                openssl-devel \
                cuda-toolkit \
                gnuplot \
                valgrind \
                perf \
                strace \
                htop \
                jq \
                curl \
                git
                
        elif command_exists pacman; then
            # Arch Linux
            info "Detected Arch Linux system"
            sudo pacman -S --noconfirm \
                base-devel \
                openssl \
                cuda \
                gnuplot \
                valgrind \
                perf \
                strace \
                htop \
                jq \
                curl \
                git
        else
            warning "Unknown Linux distribution - please install dependencies manually"
        fi
        
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if command_exists brew; then
            info "Installing dependencies via Homebrew..."
            brew install \
                openssl \
                gnuplot \
                jq \
                curl \
                git
        else
            warning "Homebrew not found - please install dependencies manually"
        fi
    fi
    
    success "System dependencies installation complete"
}

# Install Docker and container tools
install_container_tools() {
    info "Checking container tools..."
    
    if ! command_exists docker; then
        warning "Docker not found. Please install Docker manually."
        warning "Visit: https://docs.docker.com/get-docker/"
    else
        info "Docker found: $(docker --version)"
        
        # Check if user is in docker group (Linux only)
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            if ! groups | grep -q docker; then
                warning "User is not in docker group. Run: sudo usermod -aG docker \$USER"
                warning "Then log out and log back in for changes to take effect."
            fi
        fi
    fi
    
    if ! command_exists docker-compose; then
        warning "Docker Compose not found. Installing..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
            sudo chmod +x /usr/local/bin/docker-compose
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install docker-compose
        fi
    else
        info "Docker Compose found: $(docker-compose --version)"
    fi
    
    if ! command_exists kubectl; then
        info "Installing kubectl..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
            sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
            rm kubectl
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install kubectl
        fi
    else
        info "kubectl found: $(kubectl version --client --short 2>/dev/null || echo 'kubectl installed')"
    fi
}

# Install load testing tools
install_load_testing_tools() {
    info "Installing load testing tools..."
    
    if ! command_exists k6; then
        info "Installing k6..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo gpg -k
            sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
            echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
            sudo apt-get update
            sudo apt-get install k6
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install k6
        fi
    else
        info "k6 already installed: $(k6 version)"
    fi
}

# Install additional development tools
install_dev_tools() {
    info "Installing additional development tools..."
    
    # Just command runner
    if ! command_exists just; then
        info "Installing just..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install just
        fi
    else
        info "just already installed: $(just --version)"
    fi
    
    # ripgrep for fast searching
    if ! command_exists rg; then
        info "Installing ripgrep..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            if command_exists apt-get; then
                sudo apt-get install -y ripgrep
            elif command_exists yum; then
                sudo yum install -y ripgrep
            elif command_exists pacman; then
                sudo pacman -S --noconfirm ripgrep
            fi
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install ripgrep
        fi
    else
        info "ripgrep already installed"
    fi
    
    # fd for fast file finding
    if ! command_exists fd; then
        info "Installing fd..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            if command_exists apt-get; then
                sudo apt-get install -y fd-find
            elif command_exists yum; then
                sudo yum install -y fd-find
            elif command_exists pacman; then
                sudo pacman -S --noconfirm fd
            fi
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install fd
        fi
    else
        info "fd already installed"
    fi
}

# Setup Git hooks
setup_git_hooks() {
    info "Setting up Git hooks..."
    
    if [[ -d ".git" ]]; then
        # Install pre-commit hooks
        if command_exists pre-commit; then
            pre-commit install
            pre-commit install --hook-type commit-msg
            success "Pre-commit hooks installed"
        else
            warning "pre-commit not found - hooks not installed"
        fi
    else
        warning "Not in a Git repository - hooks not installed"
    fi
}

# Create necessary directories
create_directories() {
    info "Creating necessary directories..."
    
    local dirs=(
        "target/flamegraphs"
        "target/coverage"
        "target/benchmarks"
        "target/mutation-reports"
        "logs"
        "config"
        "scripts"
    )
    
    for dir in "${dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            info "Created directory: $dir"
        fi
    done
}

# Setup configuration files
setup_config() {
    info "Setting up configuration files..."
    
    # Create development config if it doesn't exist
    if [[ ! -f "config/development.toml" ]]; then
        cat > config/development.toml << 'EOF'
[server]
host = "127.0.0.1"
port = 8080
workers = 2

[logging]
level = "debug"
format = "pretty"

[encryption]
poly_modulus_degree = 8192
scale_bits = 40

[gpu]
device_id = 0
batch_size = 8

[monitoring]
metrics_port = 9090
EOF
        success "Created development configuration"
    fi
}

# Main setup function
main() {
    info "Starting FHE LLM Proxy development environment setup..."
    info "This script will install necessary tools and dependencies."
    
    # Check if running as root
    if [[ $EUID -eq 0 ]]; then
        error "This script should not be run as root"
        exit 1
    fi
    
    # Run setup steps
    check_system
    install_system_deps
    install_rust_tools
    install_python_tools
    install_container_tools
    install_load_testing_tools
    install_dev_tools
    create_directories
    setup_config
    setup_git_hooks
    
    success "Development environment setup complete!"
    info ""
    info "Next steps:"
    info "1. Restart your shell or run: source ~/.bashrc"
    info "2. If you have an NVIDIA GPU, verify CUDA: nvidia-smi"
    info "3. Build the project: just build"
    info "4. Run tests: just test"
    info "5. Start development server: just dev"
    info ""
    info "For more commands, run: just"
}

# Run main function
main "$@"