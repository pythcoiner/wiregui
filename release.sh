#!/bin/sh

# Variables
REPO="pythcoiner/wiregui"
TAG="v0.1.0"
NAME="wiregui"
RELEASE_TITLE="WireGUI $TAG"
CHANGELOG_PATH="changelog.txt"
RELEASE_NOTES=$(cat "$CHANGELOG_PATH")
BUILD_DIR="target/release"

## remove tag
#git tag -d "$TAG"
## Remove from remote repository
#git push --delete origin "$TAG"
#git push --delete github "$TAG"

# Tagging
git tag "$TAG"
git push origin "$TAG"

mv "$BUILD_DIR/$NAME" "$BUILD_DIR/$NAME-$TAG-x86_64-linux-gnu"


# Create a GitHub release
gh release create "$TAG" \
  --repo "$REPO" \
  --title "$RELEASE_TITLE" \
  --notes "$RELEASE_NOTES" \
  "$BUILD_DIR/$NAME-$TAG-x86_64-linux-gnu" \

echo "Release $TAG created successfully"
