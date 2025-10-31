# Multi-Cloud Deployment Guide

This guide covers deploying the Random Number Validator application using:
- **Google Cloud Platform (GCP)** for compute (e2-micro free tier)
- **Azure** for managed PostgreSQL database (B1ms free tier)

## Architecture

```
┌─────────────────────┐         ┌──────────────────────┐
│   GCP (us-central1) │         │  Azure (centralus)   │
│                     │         │                      │
│  ┌──────────────┐   │         │  ┌────────────────┐  │
│  │  e2-micro    │   │────────>│  │  PostgreSQL    │  │
│  │  VM Instance │   │  SSL    │  │  Flexible      │  │
│  │              │   │  5432   │  │  Server (B1ms) │  │
│  └──────────────┘   │         │  └────────────────┘  │
│                     │         │                      │
└─────────────────────┘         └──────────────────────┘
```

## Prerequisites

### 1. GCP Account Setup
- Create a Google Cloud account at https://console.cloud.google.com
- Create a new project or use an existing one
- Enable billing (free tier is available)
- Install `gcloud` CLI: https://cloud.google.com/sdk/docs/install

### 2. Azure Account Setup
- Create an Azure account at https://portal.azure.com
- Free tier includes 750 hours/month of B1ms PostgreSQL
- Install `az` CLI: https://learn.microsoft.com/en-us/cli/azure/install-azure-cli

### 3. Terraform Installation
```bash
# macOS
brew install terraform

# Linux
wget https://releases.hashicorp.com/terraform/1.6.0/terraform_1.6.0_linux_amd64.zip
unzip terraform_1.6.0_linux_amd64.zip
sudo mv terraform /usr/local/bin/
```

## Authentication Setup

### GCP Authentication
```bash
# Login to GCP
gcloud auth application-default login

# Set your project
gcloud config set project YOUR_PROJECT_ID
```

### Azure Authentication
```bash
# Login to Azure
az login

# Get your subscription ID
az account show --query id -o tsv
```

## Deployment Steps

### 1. Configure Variables

Create a `terraform.tfvars` file in the `terraform/` directory:

```hcl
# GCP Configuration
project_id     = "your-gcp-project-id"
region         = "us-central1"  # Must be us-west1, us-central1, or us-east1 for free tier
zone           = "us-central1-a"
repository_url = "https://github.com/yourusername/randomnumbervalidator.git"

# Azure Configuration
azure_subscription_id = "your-azure-subscription-id"
azure_location        = "centralus"  # Close to GCP us-central1

# Database Configuration
db_admin_username = "dbadmin"
db_admin_password = "YourSecurePassword123!"  # Min 8 chars, use strong password
```

**IMPORTANT**: Never commit `terraform.tfvars` to git. Add it to `.gitignore`.

### 2. Initialize Terraform

```bash
cd terraform/
terraform init
```

This will download the required providers:
- Google Cloud (for compute resources)
- Azure (for PostgreSQL database)
- Random (for generating unique names)

### 3. Review the Plan

```bash
terraform plan
```

Review the resources that will be created:
- **Azure Resources**:
  - Resource group
  - PostgreSQL Flexible Server (B1ms, 32GB storage)
  - Database named "randomvalidator"
  - Firewall rules (allow GCP instance IP)
- **GCP Resources**:
  - Compute instance (e2-micro)
  - Static IP address
  - Firewall rules (HTTP, SSH)

### 4. Apply Configuration

```bash
terraform apply
```

Type `yes` when prompted. This will:
1. Create Azure resource group and PostgreSQL server (~5-10 minutes)
2. Create database and configure firewall
3. Create GCP compute instance
4. Configure GCP firewall rules
5. Deploy your application with database connection

### 5. Verify Deployment

After successful deployment, Terraform will output:

```
Outputs:

instance_ip = "XX.XX.XX.XX"
database_host = "randomvalidator-db-XXXXXX.postgres.database.azure.com"
database_name = "randomvalidator"
application_url = "http://XX.XX.XX.XX:3000"
```

Visit the application URL to verify it's running.

## Cost Breakdown

### Free Tier Eligible Resources

#### GCP (Always Free)
- **e2-micro instance**: 1 instance free per month (us-west1, us-central1, us-east1)
- **30GB standard disk**: Free
- **Static IP**: $0 while in use
- **Network egress**: 1GB/month free

#### Azure (12-month free trial + Always Free)
- **PostgreSQL B1ms**: 750 hours/month free (≈31 days)
- **Storage**: 32GB free
- **Backup**: 32GB free

### Potential Costs
- GCP to Azure data transfer (after 1GB/month free): ~$0.12/GB
- If you exceed 750 hours/month on Azure DB (unlikely with single instance)
- Static IP when GCP instance is stopped: ~$0.01/hour

**Estimated monthly cost**: **$0-$5** depending on usage

## Database Access

### From Your Application
The application automatically connects using the `DATABASE_URL` environment variable set by Terraform:
```
postgresql://dbadmin:password@hostname:5432/randomvalidator?sslmode=require
```

This is configured in the systemd service at `/etc/systemd/system/randomvalidator.service`.

### Manual Connection (for debugging)

```bash
# Get connection details from Terraform
cd terraform/
terraform output database_host

# Connect using psql (from your local machine or GCP instance)
psql "postgresql://dbadmin:YourPassword@randomvalidator-db-XXXXXX.postgres.database.azure.com:5432/randomvalidator?sslmode=require"

# List tables
\dt

# Check database version
SELECT version();

# Exit
\q
```

### Using Azure Portal
1. Go to https://portal.azure.com
2. Navigate to "Azure Database for PostgreSQL flexible servers"
3. Click on your server: `randomvalidator-db-XXXXXX`
4. Use "Connect" blade for connection strings and settings

## Database Schema Management

### Using sqlx-cli for Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Set DATABASE_URL locally
export DATABASE_URL="postgresql://dbadmin:password@hostname:5432/randomvalidator?sslmode=require"

# Create a migration
sqlx migrate add create_results_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Example Migration

Create `migrations/001_create_results_table.sql`:

```sql
CREATE TABLE validation_results (
    id SERIAL PRIMARY KEY,
    input_numbers TEXT NOT NULL,
    test_results JSONB NOT NULL,
    quality_score DECIMAL(5,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_created_at ON validation_results(created_at);
```

## Monitoring & Logs

### Application Logs
```bash
# SSH into GCP instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# View live logs
sudo journalctl -u randomvalidator -f

# View last 100 lines
sudo journalctl -u randomvalidator -n 100

# View logs since specific time
sudo journalctl -u randomvalidator --since "1 hour ago"
```

### Database Monitoring (Azure Portal)
1. Go to your PostgreSQL server in Azure Portal
2. Click "Metrics" to view:
   - Active connections
   - CPU percentage
   - Storage used
   - Network I/O
3. Set up alerts for high CPU or connections

### Check Database Connection
```bash
# From GCP instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# Test database connection
psql "$DATABASE_URL" -c "SELECT 1;"

# Check active connections
psql "$DATABASE_URL" -c "SELECT count(*) FROM pg_stat_activity;"
```

## Updating the Application

### Code Changes Only
```bash
# SSH into instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# Update code
cd /opt/randomvalidator
git pull

# Rebuild
source $HOME/.cargo/env
cargo build --release --bin server

# Restart service
sudo systemctl restart randomvalidator

# Verify it's running
sudo systemctl status randomvalidator

exit
```

### Infrastructure Changes
```bash
cd terraform/

# Review changes
terraform plan

# Apply changes
terraform apply
```

### Database Schema Changes
```bash
# Create and run migration (see Database Schema Management above)
sqlx migrate add your_migration_name
# Edit the generated SQL file
sqlx migrate run
```

## Troubleshooting

### Application Won't Start

```bash
# Check service status
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo systemctl status randomvalidator"

# Check logs for errors
gcloud compute ssh randomvalidator-instance --zone=us-central1-a \
  --command="sudo journalctl -u randomvalidator -n 50"

# Common issues:
# 1. Database connection failed → check DATABASE_URL
# 2. NIST binary missing → rebuild NIST suite
# 3. Port already in use → check for other processes
```

### Database Connection Issues

```bash
# SSH into instance
gcloud compute ssh randomvalidator-instance --zone=us-central1-a

# Check if DATABASE_URL is set
sudo systemctl show randomvalidator | grep DATABASE_URL

# Test connection manually
psql "$DATABASE_URL" -c "SELECT version();"

# Common issues:
# 1. Firewall rule not allowing GCP IP
#    → Check Azure Portal firewall rules
# 2. Wrong password
#    → Verify terraform.tfvars matches
# 3. SSL required
#    → Ensure connection string has ?sslmode=require
```

### Azure Database Firewall Issues

```bash
# Get current GCP instance IP
cd terraform/
terraform output instance_ip

# Verify in Azure Portal:
# 1. Go to PostgreSQL server
# 2. Click "Networking"
# 3. Check firewall rules include GCP instance IP
# 4. Add rule manually if needed
```

### Free Tier Usage Issues

**GCP**:
```bash
# Check instance region (must be us-west1, us-central1, or us-east1)
gcloud compute instances list

# Check machine type (must be e2-micro)
gcloud compute instances describe randomvalidator-instance --zone=us-central1-a
```

**Azure**:
- Check usage: Azure Portal → Cost Management → Cost Analysis
- PostgreSQL B1ms: 750 hours/month = ~31 days (should be fine)
- Monitor monthly usage to stay within limits

### Terraform State Issues

```bash
cd terraform/

# Refresh state
terraform refresh

# View current state
terraform show

# If Azure resources exist but not in state
terraform import azurerm_resource_group.randomvalidator /subscriptions/SUB_ID/resourceGroups/randomvalidator-rg
```

## Security Best Practices

### Production Recommendations

1. **Remove wide-open database firewall rule**

   Edit `terraform/azure.tf` and remove:
   ```hcl
   resource "azurerm_postgresql_flexible_server_firewall_rule" "allow_local" {
     # Remove this entire block in production
   }
   ```

2. **Use strong passwords**
   ```bash
   # Generate strong password
   openssl rand -base64 32
   ```

3. **Enable SSL enforcement**
   Already configured with `?sslmode=require` in connection string

4. **Secrets management**
   - Store `terraform.tfvars` securely (never commit to git)
   - Consider using GCP Secret Manager or Azure Key Vault
   - Rotate passwords regularly

5. **Network security**
   - Restrict SSH access to specific IPs
   - Consider using Cloud IAP for SSH instead of public access
   - Use VPN or Private Link for database access in production

6. **Monitoring & Alerts**
   - Set up Azure Monitor alerts for database
   - Configure GCP Monitoring for compute instance
   - Enable audit logging

## Cleanup

To destroy all resources:

```bash
cd terraform/
terraform destroy
```

Type `yes` to confirm. This will delete:
- ✅ Azure PostgreSQL server and database
- ✅ Azure resource group
- ✅ GCP compute instance
- ✅ GCP static IP
- ✅ All firewall rules

**Warning**: This deletes all data permanently!

## GitHub Actions CI/CD (Optional)

To set up automated deployments:

1. Add GitHub secrets:
   - `GCP_SA_KEY`: Service account JSON
   - `AZURE_CREDENTIALS`: Azure service principal JSON
   - `DB_PASSWORD`: Database password

2. Workflows will:
   - Run tests on every push
   - Deploy to GCP on merge to main
   - Apply Terraform changes on infrastructure updates

See `.github/workflows/` for workflow definitions.

## Next Steps

- ✅ Set up database migrations with sqlx
- ✅ Implement actual validation result storage
- ✅ Add database connection pooling
- ✅ Set up automated backups (Azure Portal)
- ✅ Configure monitoring and alerts
- ✅ Add domain name and SSL certificate
- ✅ Implement proper error handling for DB connection failures

## Additional Resources

- [GCP Free Tier](https://cloud.google.com/free)
- [Azure Free Tier](https://azure.microsoft.com/en-us/pricing/free-services/)
- [Terraform GCP Provider](https://registry.terraform.io/providers/hashicorp/google/latest/docs)
- [Terraform Azure Provider](https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs)
- [sqlx Documentation](https://github.com/launchbadge/sqlx)
- [Azure PostgreSQL Docs](https://learn.microsoft.com/en-us/azure/postgresql/)

## Support

If you encounter issues:
1. Check the [Troubleshooting](#troubleshooting) section
2. Review Terraform logs: `terraform apply` output
3. Check application logs: `sudo journalctl -u randomvalidator -f`
4. Verify Azure Portal for database status
5. Check GCP Console for instance status
