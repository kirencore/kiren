# Docker Hub Setup and Troubleshooting Guide

## Issue Description
The GitHub Actions workflow is failing at the Docker Hub login step due to missing or misconfigured repository secrets.

## Root Cause
The `docker-hub.yml` workflow requires two secrets that are not properly configured:
- `DOCKERHUB_USERNAME`
- `DOCKERHUB_TOKEN`

## Solution: Configure Repository Secrets

### Step 1: Create Docker Hub Access Token
1. Go to [Docker Hub](https://hub.docker.com)
2. Sign in to your account
3. Navigate to **Account Settings** → **Security**
4. Click **New Access Token**
5. Provide a descriptive name (e.g., "GitHub Actions Kiren")
6. Set permissions to **Read, Write, Delete** (or as needed)
7. Click **Generate**
8. **Copy the token immediately** (it won't be shown again)

### Step 2: Add Secrets to GitHub Repository
1. Go to your GitHub repository: `https://github.com/kirencore/kiren`
2. Click on **Settings** tab
3. In the left sidebar, click **Secrets and variables** → **Actions**
4. Click **New repository secret**

Add these two secrets:

#### Secret 1: DOCKERHUB_USERNAME
- **Name**: `DOCKERHUB_USERNAME`
- **Value**: `mertcanaltin` (or your Docker Hub username)

#### Secret 2: DOCKERHUB_TOKEN
- **Name**: `DOCKERHUB_TOKEN`
- **Value**: The access token you created in Step 1

### Step 3: Verify Configuration
After adding the secrets, trigger the workflow by:
- Pushing to the `main` branch, or
- Creating a new release/tag

## Alternative Solutions

### Option 1: Use GitHub Container Registry Only
If you prefer not to use Docker Hub, you can disable the `docker-hub.yml` workflow and rely only on the GitHub Container Registry (GHCR) workflows which use the built-in `GITHUB_TOKEN`.

### Option 2: Update Workflow to Use Organization Secrets
If this is an organization repository, you can configure the secrets at the organization level instead.

## Troubleshooting

### Common Issues

1. **"Invalid credentials" error**
   - Verify the username is correct
   - Regenerate the Docker Hub access token
   - Ensure token has sufficient permissions

2. **"Secret not found" error**
   - Check secret names are exactly: `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN`
   - Verify secrets are added to the correct repository

3. **Workflow still failing**
   - Check if the workflow condition `github.event_name != 'pull_request'` is met
   - Verify the workflow is triggered on the correct branches

### Debug Steps
1. Check the Actions tab for detailed error messages
2. Verify secrets exist in Settings → Secrets and variables → Actions
3. Ensure Docker Hub account has permission to push to `kirencore/kiren` repository

## Current Workflow Configuration

The current workflow (`docker-hub.yml`) publishes to:
- Docker Hub repository: `kirencore/kiren`
- Tags: version-based and `latest`
- Triggers: pushes to `main`, tags starting with `v*`, and releases

## Files Involved
- `.github/workflows/docker-hub.yml` - Main Docker Hub workflow
- `publish-docker.sh` - Local Docker publishing script
- `Dockerfile` - Container build configuration

---

**Note**: This issue was identified and documented by Claude Code on 2025-08-02.