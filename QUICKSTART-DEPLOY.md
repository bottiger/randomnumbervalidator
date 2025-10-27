# Quick Start - Deploy to GCP

Get your Random Number Validator running on GCP in 10 minutes.

## Prerequisites

- GCP account ([Sign up](https://cloud.google.com) - $300 free credit)
- [gcloud CLI](https://cloud.google.com/sdk/docs/install) installed
- [Terraform](https://www.terraform.io/downloads) installed

## Step 1: Run Setup Script (5 minutes)

```bash
./scripts/setup-gcp.sh
```

This script will:
- ✅ Create GCP project configuration
- ✅ Enable required APIs
- ✅ Create service account with permissions
- ✅ Generate Terraform configuration
- ✅ Initialize Terraform

## Step 2: Deploy Infrastructure (3 minutes)

```bash
cd terraform
terraform apply
```

Type `yes` when prompted.

This creates:
- e2-micro instance (free tier)
- Static IP address
- Firewall rules
- Automatic application deployment

## Step 3: Access Your Application (2 minutes)

```bash
# Get the URL
terraform output application_url

# Example output: http://34.123.45.67:3000
```

Wait 5-10 minutes for initial deployment, then open the URL in your browser!

## Optional: Setup GitHub Actions

### 1. Push to GitHub

```bash
git add .
git commit -m "Add deployment configuration"
git push origin main
```

### 2. Add GitHub Secrets

Go to: **Repository → Settings → Secrets → Actions**

Add these 4 secrets (get values from setup script output):

| Secret | Value |
|--------|-------|
| `GCP_SA_KEY` | Contents of `~/gcp-terraform-key.json` |
| `GCP_PROJECT_ID` | Your project ID |
| `GCP_REGION` | e.g., `us-central1` |
| `GCP_ZONE` | e.g., `us-central1-a` |

### 3. Push to Deploy

Every push to `main` triggers automatic deployment!

```bash
git push origin main
```

## Verify Deployment

```bash
# Check instance status
gcloud compute instances list

# View application logs
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo journalctl -u randomvalidator -f"

# Test the API
curl -X POST http://YOUR_IP:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56,91,23,67"}'
```

## Common Issues

### "Permission denied"
```bash
# Re-authenticate
gcloud auth login
gcloud auth application-default login
```

### "Quota exceeded"
- Check you're using a free tier region: us-west1, us-central1, or us-east1
- Verify only one e2-micro instance exists

### "Port 3000 not accessible"
```bash
# Check firewall rules
gcloud compute firewall-rules list

# Verify instance is running
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo systemctl status randomvalidator"
```

## Next Steps

- 📖 Read [DEPLOYMENT.md](DEPLOYMENT.md) for detailed documentation
- 🔧 Setup GitHub Actions for CI/CD (see [.github/SETUP.md](.github/SETUP.md))
- 📊 Monitor your application
- 🎉 Share your deployment!

## Cost

**FREE** - Uses GCP Always Free tier:
- 1 e2-micro instance (1GB RAM)
- 30GB standard disk
- 1GB outbound data/month

**Estimated cost if exceeding free tier**: ~$7/month

## Cleanup

To delete all resources:

```bash
cd terraform
terraform destroy
```

Type `yes` to confirm.

## Support

- 📚 Full documentation: [DEPLOYMENT.md](DEPLOYMENT.md)
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/randomnumbervalidator/issues)
- 💬 Questions: Check troubleshooting section in DEPLOYMENT.md

---

**Success?** Star the repo and share your deployment! ⭐
