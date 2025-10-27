# GitHub Actions Setup

This file contains instructions for setting up GitHub Actions for automated deployment.

## Required GitHub Secrets

Go to your repository → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

Add the following secrets:

### 1. GCP_SA_KEY
**Description**: Service account JSON key for GCP authentication

**How to get it:**
```bash
# Create service account (if not already done)
gcloud iam service-accounts create terraform \
  --display-name="Terraform Service Account"

# Grant permissions
gcloud projects add-iam-policy-binding YOUR_PROJECT_ID \
  --member="serviceAccount:terraform@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/compute.admin"

gcloud projects add-iam-policy-binding YOUR_PROJECT_ID \
  --member="serviceAccount:terraform@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser"

# Create and download key
gcloud iam service-accounts keys create ~/gcp-key.json \
  --iam-account=terraform@YOUR_PROJECT_ID.iam.gserviceaccount.com

# Copy the ENTIRE contents of the JSON file
cat ~/gcp-key.json
```

**Secret value**: Paste the entire JSON content (everything from `{` to `}`)

---

### 2. GCP_PROJECT_ID
**Description**: Your GCP project ID

**How to get it:**
```bash
gcloud config get-value project
```

**Secret value**: Example: `randomvalidator-123456`

---

### 3. GCP_REGION
**Description**: GCP region for resources

**Secret value**: One of:
- `us-central1`
- `us-west1`
- `us-east1`

(Must be one of these for Always Free tier)

---

### 4. GCP_ZONE
**Description**: GCP zone for compute instance

**Secret value**: Examples:
- `us-central1-a`
- `us-west1-a`
- `us-east1-b`

(Must match your chosen region)

---

## Verifying Secrets

After adding all secrets, you should see 4 secrets listed:
- `GCP_SA_KEY`
- `GCP_PROJECT_ID`
- `GCP_REGION`
- `GCP_ZONE`

## Testing Workflows

### 1. Test CI Workflow
```bash
git add .
git commit -m "Add deployment configuration"
git push origin main
```

Go to **Actions** tab → **CI** workflow should start automatically

### 2. Test Terraform Workflow
```bash
# Modify something in terraform/
git add terraform/
git commit -m "Test terraform workflow"
git push origin main
```

Go to **Actions** tab → **Terraform** workflow should start automatically

### 3. Test Deploy Workflow
After infrastructure is created with Terraform, push to main:
```bash
git push origin main
```

Go to **Actions** tab → **Deploy to GCP** workflow should start automatically

## Troubleshooting

### "Error: google: could not find default credentials"
- Check that `GCP_SA_KEY` is set correctly
- Ensure the entire JSON is copied (including `{` and `}`)

### "Error: 403 - The caller does not have permission"
- Verify service account has correct roles:
  - `roles/compute.admin`
  - `roles/iam.serviceAccountUser`

### "Error: Instance not found"
- Make sure Terraform workflow has run successfully first
- Check instance exists: `gcloud compute instances list`

### Workflow not triggering
- Check `.github/workflows/` files are committed
- Verify branch name is `main` (not `master`)
- Check Actions are enabled: Settings → Actions → Allow all actions

## Manual Trigger

You can manually trigger the deployment workflow:
1. Go to **Actions** tab
2. Select **Deploy to GCP**
3. Click **Run workflow** → **Run workflow**

## Viewing Logs

After deployment:
```bash
# SSH into instance
gcloud compute ssh randomvalidator-instance --zone=YOUR_ZONE

# View application logs
sudo journalctl -u randomvalidator -f
```

## Next Steps

After successful deployment:
1. Get instance IP: `terraform output instance_ip`
2. Visit: `http://YOUR_IP:3000`
3. Test the application
4. Set up monitoring (optional)
