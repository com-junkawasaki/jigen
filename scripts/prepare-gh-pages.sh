#!/bin/bash

# Prepare docs/ directory for GitHub Pages deployment

set -e

echo "Preparing docs/ directory for GitHub Pages..."

# Create .nojekyll file to disable Jekyll processing
touch docs/.nojekyll

# Create CNAME file if GITHUB_PAGES_URL is set
if [ -n "$GITHUB_PAGES_URL" ]; then
    echo "Creating CNAME file for $GITHUB_PAGES_URL"
    echo "$GITHUB_PAGES_URL" > docs/CNAME
fi

# Create .gitkeep to ensure docs/ directory exists
touch docs/.gitkeep

echo "GitHub Pages preparation complete!"
