#!/bin/bash
set -e

echo "🚀 Random Number Validator - Deployment Script"
echo "================================================"
echo ""

# Check if terraform is installed
if ! command -v terraform &> /dev/null; then
    echo "❌ Error: Terraform is not installed."
    echo "Install it from: https://www.terraform.io/downloads"
    exit 1
fi

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo "❌ Error: gcloud CLI is not installed."
    echo "Install it from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Get current GCP project
CURRENT_PROJECT=$(gcloud config get-value project 2>/dev/null || echo "")

if [ -z "$CURRENT_PROJECT" ]; then
    echo "❌ No GCP project is configured."
    echo "Run: gcloud config set project YOUR_PROJECT_ID"
    exit 1
fi

echo "📋 Current GCP Configuration:"
echo "   Project ID: $CURRENT_PROJECT"
echo "   Account: $(gcloud config get-value account 2>/dev/null)"
echo ""

# Get repository URL from git remote
REPO_URL=$(git config --get remote.origin.url 2>/dev/null || echo "")
if [ -z "$REPO_URL" ]; then
    REPO_URL="https://github.com/bottiger/randomnumbervalidator.git"
fi

echo "📦 Repository: $REPO_URL"
echo ""

# Ask for confirmation
read -p "Deploy to project '$CURRENT_PROJECT'? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Deployment cancelled."
    exit 0
fi

echo ""
echo "🔧 Initializing Terraform..."
cd terraform

# Initialize Terraform
terraform init

echo ""
echo "📝 Planning deployment..."

# Run terraform plan
terraform plan \
    -var="project_id=$CURRENT_PROJECT" \
    -var="repository_url=$REPO_URL" \
    -out=tfplan

echo ""
echo "🚀 Applying deployment..."

# Apply the plan
terraform apply tfplan

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📊 Outputs:"
terraform output

echo ""
echo "🎉 Your application is being deployed!"
echo "It may take 5-10 minutes for the initial build to complete."
echo ""
echo "To check deployment status:"
echo "  gcloud compute instances get-serial-port-output randomvalidator-instance --zone=us-central1-a"
echo ""
echo "To SSH into the instance:"
echo "  gcloud compute ssh randomvalidator-instance --zone=us-central1-a"
echo ""
echo "To view logs:"
echo "  gcloud compute ssh randomvalidator-instance --zone=us-central1-a --command='sudo journalctl -u randomvalidator -f'"
