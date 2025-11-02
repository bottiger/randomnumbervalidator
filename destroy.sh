#!/bin/bash
set -e

echo "🗑️  Random Number Validator - Destroy Script"
echo "============================================"
echo ""
echo "⚠️  WARNING: This will permanently delete:"
echo "   - GCP Compute instance (randomvalidator-instance)"
echo "   - Static IP address"
echo "   - All firewall rules"
echo "   - All data in the database"
echo ""

# Check if terraform is installed
if ! command -v terraform &> /dev/null; then
    echo "❌ Error: Terraform is not installed."
    exit 1
fi

# Get current GCP project
CURRENT_PROJECT=$(gcloud config get-value project 2>/dev/null || echo "")

if [ -z "$CURRENT_PROJECT" ]; then
    echo "❌ No GCP project is configured."
    exit 1
fi

echo "📋 Project: $CURRENT_PROJECT"
echo ""

# Ask for confirmation
read -p "Type 'destroy' to confirm deletion: " CONFIRM
if [ "$CONFIRM" != "destroy" ]; then
    echo "Destruction cancelled."
    exit 0
fi

echo ""
echo "🔥 Destroying infrastructure..."
cd terraform

# Get repository URL from git remote
REPO_URL=$(git config --get remote.origin.url 2>/dev/null || echo "https://github.com/bottiger/randomnumbervalidator.git")

terraform destroy \
    -var="project_id=$CURRENT_PROJECT" \
    -var="repository_url=$REPO_URL"

echo ""
echo "✅ All resources destroyed."
