# Cloudflare Tunnel Setup Instructions

## Quick Setup Guide

### Step 1: Get Your Cloudflare Account ID

1. Go to https://dash.cloudflare.com
2. Log in to your account
3. Click on any domain (or "Account Home" in the left sidebar)
4. Scroll down on the right side - you'll see "Account ID"
5. **Copy this Account ID** - you'll need it for GitHub

### Step 2: Create a Cloudflare API Token

1. Go to https://dash.cloudflare.com/profile/api-tokens
2. Click **"Create Token"**
3. Click **"Create Custom Token"** (at the bottom)
4. Fill in the form:
   - **Token name**: `GitHub Actions - Cloudflare Tunnel`
   - **Permissions**:
     - Add: **Account** → **Cloudflare Tunnel** → **Edit**
     - Add: **Zone** → **DNS** → **Edit** (optional, for automatic DNS setup)
   - **Account Resources**:
     - Include → Your Account (select your account from dropdown)
   - **Zone Resources**: (if you added DNS permission)
     - Include → Specific zone → randomnumbervalidator.com
5. Click **"Continue to summary"**
6. Click **"Create Token"**
7. **COPY THE TOKEN NOW** - it won't be shown again!

### Step 3: Add Secrets to GitHub

1. Go to https://github.com/bottiger/randomnumbervalidator/settings/secrets/actions
2. Click **"New repository secret"**
3. Add the first secret:
   - Name: `CLOUDFLARE_ACCOUNT_ID`
   - Secret: Paste the Account ID from Step 1
   - Click "Add secret"
4. Click **"New repository secret"** again
5. Add the second secret:
   - Name: `CLOUDFLARE_API_TOKEN`
   - Secret: Paste the API Token from Step 2
   - Click "Add secret"

### Step 4: Re-run the Deployment

Option A - Push a change:
```bash
git push
```

Option B - Manually trigger:
1. Go to https://github.com/bottiger/randomnumbervalidator/actions
2. Click on the "Deploy to GCP" workflow
3. Click "Run workflow" button
4. Click the green "Run workflow" button

### Step 5: Verify It Works

After the workflow completes successfully:
- Check https://randomnumbervalidator.com

## Troubleshooting

### Still getting "Authentication error"?

**Check your API Token permissions:**
- The token MUST have **Account** → **Cloudflare Tunnel** → **Edit** permission
- Make sure you selected your account in "Account Resources"

**Check the Account ID:**
- Make sure you copied the full Account ID (it's a long string of letters and numbers)
- No extra spaces or characters

**Token expired or wrong?**
- Create a new token following Step 2
- Update the `CLOUDFLARE_API_TOKEN` secret in GitHub

### Domain not resolving?

The workflow will show instructions for manual DNS setup if automatic setup fails.

You'll need to add a CNAME record in Cloudflare:
- Type: CNAME
- Name: @ (or subdomain)
- Target: <TUNNEL_ID>.cfargotunnel.com
- Proxied: Yes (orange cloud)

The TUNNEL_ID will be shown in the workflow logs.
