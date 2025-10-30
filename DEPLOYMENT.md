# Deployment Guide - GCP

This guide covers deploying the Random Number Validator to Google Cloud Platform (GCP) using Terraform and GitHub Actions.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [GCP Setup](#gcp-setup)
3. [Local Terraform Deployment](#local-terraform-deployment)
4. [GitHub Actions CI/CD Setup](#github-actions-cicd-setup)
5. [Monitoring and Maintenance](#monitoring-and-maintenance)
6. [Cost Analysis](#cost-analysis)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

- **GCP Account**: Sign up at [cloud.google.com](https://cloud.google.com)
- **GCP Free Tier**: $300 credit for 90 days + Always Free tier
- **Tools**:
  - [gcloud CLI](https://cloud.google.com/sdk/docs/install)
  - [Terraform](https://www.terraform.io/downloads) (>= 1.0)
  - [Git](https://git-scm.com/downloads)

---

## GCP Setup

### 1. Create a GCP Project

```bash
# Create new project
gcloud projects create randomvalidator-PROJECT_ID --name="Random Number Validator"

# Set as default project
gcloud config set project randomvalidator-PROJECT_ID

# Enable billing (required, but free tier available)
# Go to: https://console.cloud.google.com/billing
```

### 2. Enable Required APIs

```bash
gcloud services enable compute.googleapis.com
gcloud services enable cloudresourcemanager.googleapis.com
```

### 3. Create Service Account for Terraform

```bash
# Create service account
gcloud iam service-accounts create terraform \
  --display-name="Terraform Service Account"

# Grant necessary permissions
gcloud projects add-iam-policy-binding randomvalidator-PROJECT_ID \
  --member="serviceAccount:terraform@randomvalidator-PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/compute.admin"

gcloud projects add-iam-policy-binding randomvalidator-PROJECT_ID \
  --member="serviceAccount:terraform@randomvalidator-PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser"

# Create and download key
gcloud iam service-accounts keys create ~/terraform-key.json \
  --iam-account=terraform@randomvalidator-PROJECT_ID.iam.gserviceaccount.com

# Set environment variable
export GOOGLE_APPLICATION_CREDENTIALS=~/terraform-key.json
```

---

## Local Terraform Deployment

### 1. Configure Terraform Variables

```bash
cd terraform
cp terraform.tfvars.example terraform.tfvars
```

Edit `terraform.tfvars`:

```hcl
project_id     = "randomvalidator-PROJECT_ID"
region         = "us-central1"  # Must be us-west1, us-central1, or us-east1 for free tier
zone           = "us-central1-a"
repository_url = "https://github.com/yourusername/randomnumbervalidator.git"
```

### 2. Initialize and Deploy

```bash
# Initialize Terraform
terraform init

# Preview changes
terraform plan

# Apply infrastructure
terraform apply

# Note the outputs
terraform output instance_ip
terraform output application_url
```

### 3. Access Your Application

After deployment completes (5-10 minutes for initial setup):

```bash
# Get the application URL
terraform output application_url

# Example: http://34.123.45.67:3000
```

Open the URL in your browser!

---

## GitHub Actions CI/CD Setup

### 1. Prepare GitHub Repository

```bash
# Initialize git if not already done
git init
git add .
git commit -m "Initial commit"

# Create GitHub repository and push
git remote add origin https://github.com/yourusername/randomnumbervalidator.git
git branch -M main
git push -u origin main
```

### 2. Configure GitHub Secrets

Go to your GitHub repository → Settings → Secrets and variables → Actions

Add the following secrets:

#### GCP Secrets

| Secret Name | Value | How to Get |
|-------------|-------|------------|
| `GCP_SA_KEY` | Service account JSON key | Content of `~/terraform-key.json` |
| `GCP_PROJECT_ID` | Your GCP project ID | `randomvalidator-PROJECT_ID` |
| `GCP_REGION` | GCP region | `us-central1` |
| `GCP_ZONE` | GCP zone | `us-central1-a` |

**To add `GCP_SA_KEY`:**
```bash
# Copy the entire contents of the JSON file
cat ~/terraform-key.json
# Paste into GitHub secret value (entire JSON)
```

#### Cloudflare Secrets (for Cloudflare Tunnel)

The deploy workflow sets up a Cloudflare Tunnel to provide HTTPS access to your application.

| Secret Name | Value | How to Get |
|-------------|-------|------------|
| `CLOUDFLARE_ACCOUNT_ID` | Your Cloudflare Account ID | Found in Cloudflare Dashboard → Account → Account ID |
| `CLOUDFLARE_API_TOKEN` | API token with tunnel permissions | Create in Cloudflare Dashboard → My Profile → API Tokens |

**To get your Cloudflare Account ID:**
1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com)
2. Click on any domain or go to Account Home
3. Scroll down to find your Account ID on the right side
4. Copy the Account ID value

**To create a Cloudflare API Token:**
1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com) → My Profile → API Tokens
2. Click "Create Token"
3. Use the "Create Custom Token" template
4. Set the following permissions:
   - **Account** → **Cloudflare Tunnel** → **Edit**
   - **Zone** → **DNS** → **Edit** (if using custom domain)
5. Set Account Resources: Include → Your account
6. Click "Continue to summary" → "Create Token"
7. Copy the token and add it as `CLOUDFLARE_API_TOKEN` secret in GitHub

**Optional: Custom Domain Configuration**

To use a custom domain with your Cloudflare Tunnel:

1. Add your domain to Cloudflare (if not already added)
2. Go to your GitHub repository → Settings → Variables and secrets → Variables
3. Add a new repository variable:
   - Name: `DOMAIN_NAME`
   - Value: `yourdomain.com` (or subdomain like `randomvalidator.yourdomain.com`)
4. The deploy workflow will automatically configure DNS

Without `DOMAIN_NAME` configured, your application will still be accessible via the GCP instance IP.

### 3. GitHub Actions Workflows

Three workflows are configured:

#### a) **CI Workflow** (`.github/workflows/ci.yml`)
- Runs on every push and PR
- Executes tests
- Checks formatting and linting
- Builds release binary

#### b) **Terraform Workflow** (`.github/workflows/terraform.yml`)
- Runs when `terraform/` changes
- Plans infrastructure changes
- Applies changes on merge to main
- Posts plan summary on PRs

#### c) **Deploy Workflow** (`.github/workflows/deploy.yml`)
- Runs on push to main
- SSH into instance
- Pulls latest code
- Rebuilds application
- Restarts service

### 4. Trigger Initial Deployment

```bash
# Push to main triggers deployment
git push origin main

# Or manually trigger from GitHub Actions UI
```

### 5. Monitor Deployment

Go to your repository → Actions tab to see workflow progress.

---

## Monitoring and Maintenance

### Check Application Status

```bash
# SSH into instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# Check service status
sudo systemctl status randomvalidator

# View logs
sudo journalctl -u randomvalidator -f

# Check NIST binary
cd /opt/randomvalidator/nist/sts-2.1.2/sts-2.1.2
ls -la assess

# Exit SSH
exit
```

### View Application Logs

```bash
# Stream logs
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo journalctl -u randomvalidator -f"

# View recent logs
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo journalctl -u randomvalidator -n 100"
```

### Manual Deployment

```bash
# SSH into instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# Update code
cd /opt/randomvalidator
git pull

# Rebuild NIST
cd nist/sts-2.1.2/sts-2.1.2
make clean && make

# Rebuild application
cd /opt/randomvalidator
source $HOME/.cargo/env
cargo build --release --bin server

# Restart service
sudo systemctl restart randomvalidator

# Exit
exit
```

### Update Instance Configuration

```bash
# Modify terraform/main.tf or variables
cd terraform

# Plan changes
terraform plan

# Apply changes
terraform apply
```

---

## Cost Analysis

### Free Tier Limits (Always Free)

- **Compute**: 1 e2-micro instance in us-west1, us-central1, or us-east1
- **Storage**: 30GB standard persistent disk
- **Network**: 1GB egress per month (North America)
- **Static IP**: Free while in use

### Estimated Costs (After Free Tier)

| Resource | Free Tier | After Free Tier |
|----------|-----------|-----------------|
| e2-micro instance | Free | ~$7/month |
| 30GB disk | Free | ~$2/month |
| Static IP | Free | ~$3/month |
| Egress (>1GB) | 1GB free | $0.12/GB |

**Total if staying in free tier**: $0/month ✅

---

## Troubleshooting

### Application Not Accessible

```bash
# Check firewall rules
gcloud compute firewall-rules list

# Verify instance is running
gcloud compute instances list

# Check if service is running
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo systemctl status randomvalidator"
```

### NIST Tests Failing

```bash
# SSH into instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# Check NIST binary
cd /opt/randomvalidator/nist/sts-2.1.2/sts-2.1.2
ls -la assess

# Rebuild if needed
make clean && make

# Restart service
sudo systemctl restart randomvalidator
exit
```

### Deployment Failed

```bash
# Check GitHub Actions logs in repository

# Manually trigger startup script
gcloud compute instances add-metadata randomvalidator-instance \
  --zone=us-central1-a \
  --metadata=startup-script="$(cat terraform/startup-script.sh)"

gcloud compute instances reset randomvalidator-instance --zone=us-central1-a
```

### Terraform State Issues

```bash
# If terraform state gets out of sync
cd terraform

# Refresh state
terraform refresh

# Import existing resources if needed
terraform import google_compute_instance.randomvalidator randomvalidator-instance
```

### SSH Access Issues

```bash
# Add SSH key
gcloud compute config-ssh

# Or use browser-based SSH
# Go to: https://console.cloud.google.com/compute/instances
# Click "SSH" button next to instance
```

---

## Cleanup / Destroy Infrastructure

**Warning**: This will delete all resources!

```bash
cd terraform

# Preview what will be destroyed
terraform plan -destroy

# Destroy all resources
terraform destroy

# Confirm by typing: yes
```

---

## Additional Resources

- [GCP Free Tier](https://cloud.google.com/free)
- [Terraform GCP Provider](https://registry.terraform.io/providers/hashicorp/google/latest/docs)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GCP Compute Engine Docs](https://cloud.google.com/compute/docs)

---

## Support

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section
2. Review GitHub Actions logs
3. Check GCP console logs
4. Open an issue on GitHub
