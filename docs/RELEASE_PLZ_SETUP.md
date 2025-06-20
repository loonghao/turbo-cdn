# Release-plz Setup Guide

## Problem Description

If you're encountering this error in GitHub Actions:

```
Error: fatal: could not read Username for 'https://github.com': terminal prompts disabled
```

This indicates an authentication issue with the release-plz workflow.

## Solution

The workflow now supports multiple authentication methods with automatic fallback:

### Option 1: Use GitHub App (Recommended - Most Secure)

The workflow now supports GitHub App authentication with automatic fallback. This is the most secure and reliable method.

#### Setup GitHub App:

1. **Create GitHub App**:
   - Go to GitHub Settings → Developer settings → GitHub Apps
   - Click "New GitHub App"
   - Set these fields:
     - **GitHub App name**: `release-plz-bot` (or any name you prefer)
     - **Homepage URL**: Your GitHub profile or repository URL
     - **Webhook**: Uncheck "Active" (not needed)
   - **Repository permissions**:
     - Contents: Read & write
     - Pull requests: Read & write
     - Actions: Read (optional, for better workflow integration)
   - **Where can this GitHub App be installed?**: "Only on this account"

2. **Generate Private Key**:
   - After creating the app, go to the app settings
   - Scroll down to "Private keys" section
   - Click "Generate a private key"
   - Download and securely store the `.pem` file

3. **Install the App**:
   - Go to "Install App" tab in the GitHub App settings
   - Install it on your repository

4. **Add Secrets to Repository**:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Add these secrets:
     - `APP_ID`: Your GitHub App ID (found in app settings)
     - `APP_PRIVATE_KEY`: Content of the `.pem` file you downloaded

### Option 2: Use Personal Access Token (PAT)

If you need additional permissions (e.g., to trigger other workflows), create a PAT:

1. **Create PAT**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Set expiration (recommend 90 days or 1 year)
   - Select these scopes:
     - `repo` (Full control of private repositories)
     - `workflow` (Update GitHub Action workflows)
     - `write:packages` (Upload packages to GitHub Package Registry)

2. **Add PAT to Repository Secrets**:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `RELEASE_PLZ_TOKEN`
   - Value: Your PAT token

3. **Add Cargo Registry Token** (for crates.io publishing):
   - Get your token from https://crates.io/me
   - Add as repository secret named `CARGO_REGISTRY_TOKEN`

## Verification

After making these changes:

1. Push a commit to the main branch
2. Check the Actions tab to see if the release-plz workflow runs successfully
3. If it still fails, check the logs for specific error messages

## Troubleshooting

### Common Issues:

1. **Token Expired**: Regenerate the PAT and update the secret
2. **Insufficient Permissions**: Ensure the PAT has `repo` and `workflow` scopes
3. **Repository Settings**: Check that Actions have write permissions in repository settings

### Repository Settings to Check:

1. Go to Settings → Actions → General
2. Ensure "Workflow permissions" is set to "Read and write permissions"
3. Check "Allow GitHub Actions to create and approve pull requests"

### Option 3: Use GITHUB_TOKEN (Basic - Limited functionality)

The default `GITHUB_TOKEN` works for basic operations but cannot trigger other workflows.

## Authentication Priority

The workflow uses this authentication priority:
1. **GitHub App token** (if `APP_ID` and `APP_PRIVATE_KEY` secrets are configured)
2. **Personal Access Token** (if `RELEASE_PLZ_TOKEN` secret is configured)
3. **Default GITHUB_TOKEN** (fallback)

## What Was Fixed

The workflow configuration has been updated with:

1. **GitHub App Support**: Added automatic GitHub App token generation with fallback
2. **Enhanced Authentication**: Multiple authentication methods with priority order
3. **Latest Action Version**: Updated to release-plz action v0.5.108
4. **Improved Error Handling**: Continue-on-error for GitHub App token generation
5. **Proper Token Handling**: Consistent token usage across all steps

These changes provide a robust authentication system that should resolve all authentication issues.
