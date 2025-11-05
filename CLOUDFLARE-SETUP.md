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
3. ✅ Configure the tunnel to proxy `localhost:3000` (not publicly exposed)
4. ✅ Create a DNS CNAME record pointing to the tunnel
5. ✅ Enable Cloudflare's free SSL (automatic)
6. ✅ Enable Cloudflare's CDN, DDoS protection, and caching
7. ✅ **Enforce TLS-only access** - HTTP ports are not publicly exposed
8. ✅ **Automatic HTTP to HTTPS redirect** via Cloudflare (see configuration below)

## Cloudflare SSL/TLS Configuration (HTTPS Enforcement)

To ensure all traffic uses HTTPS and HTTP requests are automatically redirected:

### Step 1: Configure SSL/TLS Mode

1. Go to your domain in Cloudflare Dashboard: https://dash.cloudflare.com
2. Navigate to **SSL/TLS** → **Overview**
3. Set the encryption mode to **Full** (recommended)
   - **Full**: Encrypts traffic between visitors and Cloudflare, and between Cloudflare and your origin server
   - The origin server (your app on port 3000) uses HTTP, but it's only accessible via localhost (not publicly exposed)

### Step 2: Enable Always Use HTTPS

1. In the same SSL/TLS section, go to **Edge Certificates**
2. Scroll down to **Always Use HTTPS**
3. Toggle it to **On**
4. This ensures all HTTP requests are automatically redirected to HTTPS

### Step 3: Enable HSTS (Optional but Recommended)

1. Scroll to **HTTP Strict Transport Security (HSTS)**
2. Click **Enable HSTS**
3. Configure the settings:
   - Max Age Header: 6 months (recommended)
   - Apply HSTS policy to subdomains: On (if using subdomains)
   - No-Sniff Header: On
   - Preload: Off (only enable if you want to be added to browser preload lists)
4. Click **Next** and **I understand** to confirm

### Security Benefits

With this configuration:
- ✅ All traffic is encrypted end-to-end
- ✅ HTTP requests automatically redirect to HTTPS
- ✅ Port 3000 is only accessible via localhost (not publicly exposed)
- ✅ Cloudflare provides TLS termination with automatic certificate renewal
- ✅ HSTS prevents downgrade attacks
- ✅ No direct IP access to the application

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
✅ **TLS-only access** - HTTP ports not publicly exposed
✅ **Automatic HTTP to HTTPS redirect**
✅ Cloudflare CDN (faster global access)
✅ DDoS protection
✅ No exposed ports (port 3000 is only accessible via localhost)
✅ Automatic failover and health checks
✅ No need to manage certificates
✅ HSTS support for additional security
✅ Protection against man-in-the-middle attacks
