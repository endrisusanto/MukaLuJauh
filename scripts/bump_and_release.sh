#!/usr/bin/env bash
set -e

# ==============================================================================
# MukaLuJauh - Auto Commit, Build, Version Bump & GitHub Release Script
# Target Repository: https://github.com/endrisusanto/MukaLuJauh
# ==============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo -e "${BLUE}======================================================${NC}"
echo -e "${BLUE}       🚀 MukaLuJauh Release Automation & CI/CD       ${NC}"
echo -e "${BLUE}======================================================${NC}"

# 1. Check Cargo.toml presence
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found in $PROJECT_ROOT${NC}"
    exit 1
fi

# 2. Extract current version
CURRENT_VERSION=$(grep -m1 '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
echo -e "Current version: ${YELLOW}v${CURRENT_VERSION}${NC}"

# 3. Parse bump argument (patch, minor, major, or specific version)
BUMP_TYPE="${1:-patch}"

IFS='.' read -r -a VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR="${VERSION_PARTS[0]}"
MINOR="${VERSION_PARTS[1]}"
PATCH="${VERSION_PARTS[2]}"

case "$BUMP_TYPE" in
    patch)
        PATCH=$((PATCH + 1))
        NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
        ;;
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
        ;;
    v* | [0-9]*)
        NEW_VERSION="${BUMP_TYPE#v}"
        ;;
    *)
        echo -e "${RED}Invalid bump type: $BUMP_TYPE. Use patch | minor | major | <x.y.z>${NC}"
        exit 1
        ;;
esac

TAG_NAME="v${NEW_VERSION}"
echo -e "Target release version: ${GREEN}${TAG_NAME}${NC}"

# 4. Optional custom commit message
COMMIT_MSG="${2:-chore(release): bump version to ${TAG_NAME}}"

# 5. Update version in Cargo.toml
echo -e "\n${BLUE}📝 Updating Cargo.toml version...${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' -E "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
else
    sed -i -E "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
fi

# 6. Run Cargo checks and update Cargo.lock
echo -e "${BLUE}🔨 Verifying build and updating Cargo.lock...${NC}"
cargo check
cargo test --quiet || true

# 7. Check / Initialize Git Repo
if [ ! -d ".git" ]; then
    echo -e "${YELLOW}Initialising Git repository...${NC}"
    git init
    git branch -M main
    git remote add origin https://github.com/endrisusanto/MukaLuJauh.git || true
else
    # Ensure origin remote is configured
    if ! git remote | grep -q 'origin'; then
        git remote add origin https://github.com/endrisusanto/MukaLuJauh.git
    fi
fi

# 8. Stage and Commit
echo -e "${BLUE}📦 Staging files and creating commit...${NC}"
git add .
git commit -m "$COMMIT_MSG" || {
    echo -e "${YELLOW}No new changes to commit (already up to date).${NC}"
}

# 9. Create Tag
echo -e "${BLUE}🏷️  Creating tag ${TAG_NAME}...${NC}"
git tag -a "$TAG_NAME" -m "Release ${TAG_NAME}" -f

# 10. Push to GitHub
echo -e "\n${GREEN}🚀 Pushing commit & tag to GitHub repository!${NC}"
echo -e "Remote: ${YELLOW}https://github.com/endrisusanto/MukaLuJauh${NC}"
echo -e "Command: ${BLUE}git push origin main --tags${NC}\n"

if [ -n "$CI" ] || [ "$AUTO_PUSH" = "true" ] || [ ! -t 0 ]; then
    REPLY="y"
else
    read -p "Push to GitHub now? (y/N): " -n 1 -r
    echo
fi

if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push origin main --tags || git push origin HEAD --tags
    echo -e "\n${GREEN}🎉 Successfully pushed! GitHub Actions CI/CD has been triggered.${NC}"
    echo -e "Check release workflow at: ${BLUE}https://github.com/endrisusanto/MukaLuJauh/actions${NC}"
else
    echo -e "${YELLOW}Skipped git push. Run manually when ready:${NC}"
    echo -e "  git push origin main --tags"
fi
