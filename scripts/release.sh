#!/bin/bash

# Kiren Release Script
# Usage: ./scripts/release.sh [version]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

VERSION=${1:-}

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Validate version format
validate_version() {
    if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        error "Version must be in format X.Y.Z (e.g., 0.1.0)"
    fi
}

# Check git status
check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        error "Working directory is not clean. Commit or stash changes first."
    fi
    
    if [[ $(git branch --show-current) != "main" ]]; then
        warn "Not on main branch. Current branch: $(git branch --show-current)"
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Update version in Cargo.toml
update_version() {
    log "Updating version to $VERSION..."
    sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    rm Cargo.toml.bak
    success "Version updated in Cargo.toml"
}

# Update CHANGELOG.md
update_changelog() {
    log "Updating CHANGELOG.md..."
    
    local date=$(date +%Y-%m-%d)
    local temp_file=$(mktemp)
    
    # Create new changelog entry
    cat > "$temp_file" << EOF
# Changelog

## [${VERSION}] - ${date}

### Added
- TODO: Add release notes here

### Changed
- TODO: Add changes here

### Fixed
- TODO: Add fixes here

EOF
    
    # Append existing changelog (skip first line if it's "# Changelog")
    if [[ -f CHANGELOG.md ]]; then
        tail -n +2 CHANGELOG.md >> "$temp_file"
    fi
    
    mv "$temp_file" CHANGELOG.md
    
    success "CHANGELOG.md updated"
    warn "Please edit CHANGELOG.md to add release notes before continuing"
    read -p "Press Enter when ready to continue..."
}

# Run tests and checks
run_checks() {
    log "Running tests and checks..."
    
    cargo fmt --check || error "Code is not formatted. Run 'cargo fmt'"
    cargo clippy -- -D warnings || error "Clippy checks failed"
    cargo test || error "Tests failed"
    
    success "All checks passed"
}

# Build release
build_release() {
    log "Building release binary..."
    cargo build --release
    
    if [[ -f "target/release/kiren" ]]; then
        local size=$(ls -lh target/release/kiren | awk '{print $5}')
        success "Release binary built (size: $size)"
    else
        error "Release binary not found"
    fi
}

# Create git tag
create_tag() {
    log "Creating git tag v$VERSION..."
    
    git add Cargo.toml CHANGELOG.md
    git commit -m "Release v$VERSION"
    git tag -a "v$VERSION" -m "Release v$VERSION"
    
    success "Git tag v$VERSION created"
}

# Push to GitHub
push_release() {
    log "Pushing to GitHub..."
    
    git push origin main
    git push origin "v$VERSION"
    
    success "Pushed to GitHub"
    log "GitHub Actions will build and create the release automatically"
    log "Monitor progress at: https://github.com/mertcanaltin/kiren/actions"
}

# Main release process
main() {
    echo "🚀 Kiren Release Process"
    echo "========================"
    echo ""
    
    if [[ -z "$VERSION" ]]; then
        echo "Usage: $0 <version>"
        echo "Example: $0 0.2.0"
        exit 1
    fi
    
    validate_version
    check_git_status
    
    log "Preparing release v$VERSION..."
    echo ""
    
    update_version
    update_changelog
    run_checks
    build_release
    
    echo ""
    log "Ready to create release. This will:"
    log "1. Commit version changes"
    log "2. Create git tag v$VERSION"
    log "3. Push to GitHub"
    log "4. Trigger automated builds"
    echo ""
    
    read -p "Continue with release? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Release cancelled"
        exit 0
    fi
    
    create_tag
    push_release
    
    echo ""
    success "🎉 Release v$VERSION initiated!"
    echo ""
    log "Next steps:"
    log "1. Monitor GitHub Actions: https://github.com/mertcanaltin/kiren/actions"
    log "2. Update release notes on GitHub when builds complete"
    log "3. Announce release on social media"
    log "4. Update package managers (Homebrew, etc.)"
}

main "$@"