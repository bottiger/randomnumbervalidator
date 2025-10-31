# Cloudflare Tunnel Setup Guide

This guide will help you set up a Cloudflare Tunnel with free SSL for your domain.

## Prerequisites

✅ You've already added to GitHub Secrets:
- `CLOUDFLARE_ACCOUNT_ID`
- `CLOUDFLARE_API_TOKEN`

## Step 1: Verify API Token Permissions

Make sure your Cloudflare API token has these permissions:

1. Go to: https://dash.cloudflare.com/profile/api-tokens
2. Find your token and verify it has:
   - **Account** → **Cloudflare Tunnel** → **Edit**
   - **Zone** → **DNS** → **Edit**
   - **Zone** → **Zone** → **Read**

If missing permissions, create a new token with these permissions.

## Step 2: Add Domain Name Variable

1. Go to your GitHub repository
2. Navigate to: **Settings → Secrets and variables → Actions → Variables tab**
3. Click **New repository variable**
4. Add:
   - **Name**: `DOMAIN_NAME`
   - **Value**: `your-domain.com` (or `subdomain.your-domain.com`)

## Step 3: Ensure Domain is in Cloudflare

Make sure your domain is added to Cloudflare and nameservers are pointed to Cloudflare.

To verify:
1. Go to: https://dash.cloudflare.com
2. Find your domain in the list
3. Check that status is **Active**

## Step 4: Deploy

Commit and push your changes, or manually trigger the workflow:

```bash
git add .
git commit -m "Add Cloudflare Tunnel support"
git push origin main
```

Or go to **Actions** tab and click **Run workflow** on "Deploy to GCP".

## Step 5: Verify

After deployment completes (2-3 minutes):

1. Check the Actions log for:
   ```
   ✅ Domain is accessible at https://your-domain.com
   ```

2. Visit your domain:
   ```
   https://your-domain.com
   ```

3. You should see your Random Number Validator with a valid SSL certificate!

**Important**: The deployment now automatically validates your domain. If the domain is not accessible within 60 seconds, the deployment will fail with a clear error message. This ensures you know immediately if there's a configuration problem.

If validation fails, the error message will include:
- Possible reasons for failure
- Commands to debug the issue
- Links to troubleshooting steps

## What This Does

The deployment will:
1. ✅ Create a Cloudflare Tunnel (if doesn't exist)
2. ✅ Install `cloudflared` on your GCP instance
3. ✅ Configure the tunnel to proxy `localhost:3000`
4. ✅ Create a DNS CNAME record pointing to the tunnel
5. ✅ Enable Cloudflare's free SSL (automatic)
6. ✅ Enable Cloudflare's CDN, DDoS protection, and caching

## Troubleshooting

### Domain validation fails during deployment

If you see: `❌ Error: Domain https://your-domain.com is not accessible after 60 seconds`

This usually means:

1. **Domain not in Cloudflare**: Make sure you've added your domain to Cloudflare and nameservers are pointed correctly
2. **DNS propagation delay**: First deployment may take longer. Wait 5 minutes and re-run the workflow
3. **Tunnel not running**: SSH to instance and check:
   ```bash
   gcloud compute ssh randomvalidator-instance --zone=YOUR_ZONE \
     --command="sudo systemctl status cloudflared"
   ```

### DNS not resolving
- Wait 5-10 minutes for DNS propagation (especially on first setup)
- Check Cloudflare dashboard → DNS → Records for the CNAME entry
- Verify domain status is **Active** in Cloudflare

### 502 Bad Gateway
- Check tunnel status: `sudo systemctl status cloudflared`
- Check application status: `sudo systemctl status randomvalidator`
- View logs: `sudo journalctl -u cloudflared -n 50`
- Restart: `sudo systemctl restart cloudflared`

### Tunnel not created
- Verify API token permissions (see Step 1)
- Check GitHub Actions logs for detailed error messages
- Ensure `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` secrets are set correctly

### Domain not found in Cloudflare account

If you see: `❌ Error: Domain example.com not found in your Cloudflare account`

1. Go to https://dash.cloudflare.com
2. Click **Add site** and follow the wizard
3. Update your domain's nameservers to Cloudflare's nameservers
4. Wait for status to show **Active**
5. Re-run the GitHub Actions workflow

## Manual DNS Setup (if automated setup fails)

If the automated DNS setup fails, add manually in Cloudflare:

1. Go to your domain in Cloudflare Dashboard
2. Click **DNS** → **Records**
3. Click **Add record**
4. Add:
   - **Type**: CNAME
   - **Name**: @ (for root domain) or subdomain name
   - **Target**: `<TUNNEL_ID>.cfargotunnel.com` (find in Actions log)
   - **Proxy status**: Proxied (orange cloud)
5. Click **Save**

## Benefits

✅ Free SSL certificate (automatic renewal)
✅ Cloudflare CDN (faster global access)
✅ DDoS protection
✅ No exposed ports (port 3000 is only accessible via tunnel)
✅ Automatic failover and health checks
✅ No need to manage certificates
