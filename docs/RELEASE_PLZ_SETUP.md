# Release-plz Setup Guide

## Problem Description

If you're encountering this error in GitHub Actions:

```
Error: fatal: could not read Username for 'https://github.com': terminal prompts disabled
```

This indicates an authentication issue with the release-plz workflow.

## Solution

### Option 1: Use GITHUB_TOKEN (Recommended for most cases)

The default `GITHUB_TOKEN` should work for most release-plz operations. The workflow has been updated to include proper permissions and configuration.

### Option 2: Create a Personal Access Token (PAT)

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

## What Was Fixed

The workflow configuration has been updated with:

1. Added `persist-credentials: true` to checkout steps
2. Enhanced permissions for the workflow
3. Proper token handling in both release and PR jobs
4. Improved error handling and authentication flow

These changes should resolve the authentication issues and allow release-plz to work properly.
