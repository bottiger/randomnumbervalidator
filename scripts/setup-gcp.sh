#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Random Number Validator - GCP Setup${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo -e "${RED}Error: gcloud CLI not found${NC}"
    echo "Please install: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Check if terraform is installed
if ! command -v terraform &> /dev/null; then
    echo -e "${RED}Error: terraform not found${NC}"
    echo "Please install: https://www.terraform.io/downloads"
    exit 1
fi

# Get project ID
echo -e "${YELLOW}Enter your GCP Project ID:${NC}"
read -r PROJECT_ID

if [ -z "$PROJECT_ID" ]; then
    echo -e "${RED}Error: Project ID cannot be empty${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Select region (must be one of these for free tier):${NC}"
echo "1) us-central1 (Iowa)"
echo "2) us-west1 (Oregon)"
echo "3) us-east1 (South Carolina)"
read -r REGION_CHOICE

case $REGION_CHOICE in
    1) REGION="us-central1"; ZONE="us-central1-a" ;;
    2) REGION="us-west1"; ZONE="us-west1-a" ;;
    3) REGION="us-east1"; ZONE="us-east1-b" ;;
    *) echo -e "${RED}Invalid choice${NC}"; exit 1 ;;
esac

echo ""
echo -e "${YELLOW}Enter your GitHub repository URL:${NC}"
echo "(e.g., https://github.com/username/randomnumbervalidator.git)"
read -r REPO_URL

if [ -z "$REPO_URL" ]; then
    echo -e "${RED}Error: Repository URL cannot be empty${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Configuration:${NC}"
echo "  Project ID: $PROJECT_ID"
echo "  Region: $REGION"
echo "  Zone: $ZONE"
echo "  Repository: $REPO_URL"
echo ""
echo -e "${YELLOW}Continue? (y/n)${NC}"
read -r CONFIRM

if [ "$CONFIRM" != "y" ]; then
    echo "Aborted."
    exit 0
fi

# Set project
echo ""
echo -e "${GREEN}Setting GCP project...${NC}"
gcloud config set project "$PROJECT_ID"

# Enable APIs
echo ""
echo -e "${GREEN}Enabling required APIs...${NC}"
gcloud services enable compute.googleapis.com
gcloud services enable cloudresourcemanager.googleapis.com

# Create service account
echo ""
echo -e "${GREEN}Creating service account...${NC}"
SA_NAME="terraform"
SA_EMAIL="${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

# Check if service account exists
if gcloud iam service-accounts describe "$SA_EMAIL" &> /dev/null; then
    echo "Service account already exists"
else
    gcloud iam service-accounts create "$SA_NAME" \
        --display-name="Terraform Service Account"
fi

# Grant permissions
echo ""
echo -e "${GREEN}Granting permissions...${NC}"
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="roles/compute.admin" \
    --quiet

gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="roles/iam.serviceAccountUser" \
    --quiet

# Create key
echo ""
echo -e "${GREEN}Creating service account key...${NC}"
KEY_FILE="$HOME/gcp-terraform-key.json"
gcloud iam service-accounts keys create "$KEY_FILE" \
    --iam-account="$SA_EMAIL"

echo -e "${GREEN}Key saved to: ${KEY_FILE}${NC}"

# Create terraform.tfvars
echo ""
echo -e "${GREEN}Creating Terraform configuration...${NC}"
cd terraform

cat > terraform.tfvars <<EOF
project_id     = "$PROJECT_ID"
region         = "$REGION"
zone           = "$ZONE"
repository_url = "$REPO_URL"
EOF

echo -e "${GREEN}terraform.tfvars created${NC}"

# Set environment variable
export GOOGLE_APPLICATION_CREDENTIALS="$KEY_FILE"

# Initialize Terraform
echo ""
echo -e "${GREEN}Initializing Terraform...${NC}"
terraform init

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""
echo "1. Review the Terraform plan:"
echo "   cd terraform && terraform plan"
echo ""
echo "2. Deploy infrastructure:"
echo "   terraform apply"
echo ""
echo "3. For GitHub Actions, add these secrets to your repository:"
echo "   - GCP_SA_KEY: $(cat "$KEY_FILE")"
echo "   - GCP_PROJECT_ID: $PROJECT_ID"
echo "   - GCP_REGION: $REGION"
echo "   - GCP_ZONE: $ZONE"
echo ""
echo -e "${YELLOW}Service account key location:${NC} $KEY_FILE"
echo -e "${YELLOW}Keep this file secure and never commit it to git!${NC}"
