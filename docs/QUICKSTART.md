# CCM Rust - Quick Start Guide

## Installation

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Platform Dependencies**:

   **macOS**:
   - No additional dependencies needed

   **Ubuntu/Debian**:
   ```bash
   sudo apt-get install libsecret-1-dev
   ```

   **Fedora**:
   ```bash
   sudo dnf install libsecret-devel
   ```

   **Windows**: No additional dependencies needed

### Build from Source

```bash
cd ccm_rust

# Development build
cargo build

# Release build (optimized)
cargo build --release

# The binary will be at:
# - target/debug/ccm (dev)
# - target/release/ccm (release)
```

### Install Globally

```bash
cargo install --path .
```

## First Time Setup

### 1. Initialize and Set PIN

```bash
ccm auth set
```

You'll be prompted to enter a new PIN (minimum 4 characters).

### 2. Add Your First Entry

CCM v0.9.1+ uses a unified entry model with environment variable mappings:

#### API Configuration
```bash
ccm add claude "sk-ant-api03-..." \
  --env ANTHROPIC_API_KEY=SECRET \
  --env ANTHROPIC_BASE_URL=https://api.anthropic.com \
  --env ANTHROPIC_MODEL=claude-sonnet-4-20250514
```

#### Password
```bash
ccm add github "my-secret-password" \
  --env GITHUB_TOKEN=SECRET \
  --notes "Personal GitHub token"
```

#### Configuration
```bash
ccm add prod-db "db-password-123" \
  --env DB_HOST=prod-db.example.com \
  --env DB_PASSWORD=SECRET \
  --env DB_PORT=5432 \
  --tags production
```

## Common Commands

### List All Entries
```bash
ccm list
```

### List with Details
```bash
ccm list --verbose
ccm list --json
```

### Get Entry with Secret
```bash
ccm get claude
```

### Copy Secret to Clipboard
```bash
ccm get claude -c
```

### Search Entries
```bash
ccm search claude
ccm search github
```

### Use Entry (Set Environment Variables)
```bash
ccm use claude
```

This sets all environment variables defined in the entry, substituting `SECRET` with the actual decrypted value.

### Update Entry
```bash
ccm update claude \
  --env ANTHROPIC_API_KEY=SECRET \
  --notes "Updated API key"
```

### Delete Entry
```bash
ccm delete claude

# Delete multiple entries
ccm delete entry1 entry2 entry3

# Skip confirmation
ccm delete claude --force
```

### Show Statistics
```bash
ccm stats
```

## Authentication Management

### Set PIN
```bash
ccm auth set
```

### Change PIN
```bash
ccm auth change
```

### Remove PIN
```bash
ccm auth remove
```

### Login (if not authenticated)
```bash
ccm auth on
```

### Logout
```bash
ccm auth off
```

## Environment Variables

### How It Works

When you run `ccm use <entry>`, CCM sets environment variables based on the entry's metadata. The `SECRET` placeholder is replaced with the actual decrypted secret value.

#### Example Entry:
```bash
ccm add my-app "my-secret-key" \
  --env APP_API_KEY=SECRET \
  --env APP_BASE_URL=https://api.example.com \
  --env APP_TIMEOUT=30
```

Stored metadata:
- `APP_API_KEY` → `"SECRET"` (placeholder)
- `APP_BASE_URL` → `"https://api.example.com"`
- `APP_TIMEOUT` → `"30"`

When you run `ccm use my-app`:
- `APP_API_KEY` = `"my-secret-key"` (decrypted)
- `APP_BASE_URL` = `"https://api.example.com"`
- `APP_TIMEOUT` = `30`

### Platform Differences

**Windows**: Uses `setx` to set user-level environment variables (requires terminal restart)

**Unix/macOS**: Modifies shell config files (~/.zshrc, ~/.bashrc, ~/.config/fish/config.fish)

## Common Patterns

### Claude API
```bash
ccm add claude "sk-ant-xxx" \
  --env ANTHROPIC_API_KEY=SECRET \
  --env ANTHROPIC_BASE_URL=https://api.anthropic.com \
  --env ANTHROPIC_MODEL=claude-sonnet-4-20250514
```

### OpenAI API
```bash
ccm add openai "sk-xxx" \
  --env OPENAI_API_KEY=SECRET \
  --env OPENAI_BASE_URL=https://api.openai.com \
  --env OPENAI_MODEL=gpt-4
```

### Gemini API
```bash
ccm add gemini "some-key" \
  --env GEMINI_API_KEY=SECRET \
  --env GEMINI_BASE_URL=https://generativelanguage.googleapis.com
```

### GitHub Token
```bash
ccm add github "ghp_xxx" \
  --env GITHUB_TOKEN=SECRET \
  --notes "Personal GitHub token"
```

### With Tags and Notes
```bash
ccm add prod-api "sk-xxx" \
  --env API_KEY=SECRET \
  --env BASE_URL=https://api.example.com \
  --tags production,api \
  --notes "Production API key - expires 2025-12-31"
```

### Database Configuration
```bash
ccm add prod-db "db-password-123" \
  --env DB_HOST=prod-db.example.com \
  --env DB_PASSWORD=SECRET \
  --env DB_PORT=5432 \
  --env DB_NAME=production \
  --tags production
```

## Advanced Usage

### Tags
```bash
# Add entry with tags (comma-separated)
ccm add my-api "sk-xxx" \
  --env API_KEY=SECRET \
  --tags work,production

# View tags in verbose output
ccm list --verbose
```

### Notes
```bash
# Add entry with notes
ccm add personal-api "sk-xxx" \
  --env API_KEY=SECRET \
  --notes "Personal API, don't use for work projects"
```

### Import and Export
```bash
# Import from CSV or JSON
ccm import passwords.csv
ccm import backup.json

# Export to encrypted backup
ccm export

# Export specific entry
ccm export claude

# Plaintext export (use with caution!)
ccm export -d
```

## Debug Mode

Enable debug logging:
```bash
DEBUG=1 ccm list
DEBUG=1 ccm get claude
```

## Help

### General Help
```bash
ccm help
```

### Command-Specific Help
```bash
ccm help add
ccm help get
ccm help list
```

## Security Best Practices

1. **Use Strong PINs**: At least 6-8 characters
2. **Backup Your Database**: The database is at `~/.ccm/ccm.db`
3. **Don't Share PINs**: PIN loss = permanent data loss (by design)
4. **Use OS Security**: Lock your screen when away
5. **Audit Regularly**: Review entries with `ccm list --verbose`

## Troubleshooting

### "OS secret service is required"
- Ensure your OS keyring is available
- Windows: Check Credential Manager
- macOS: Check Keychain Access
- Linux: Ensure gnome-keyring or similar is running

### "PIN is required"
- You need to authenticate first: `ccm auth on`
- Or set a PIN: `ccm auth set`

### "Master key not available"
- Ensure you've authenticated: `ccm auth on`
- Check debug output: `DEBUG=1 ccm list`

### Database Locked
- Only one CCM process can access the database at a time
- Check for other running CCM processes

## Migration from v0.9.0

### Breaking Changes

**Old syntax (v0.9.0)**:
```bash
ccm add api claude sk-ant-xxx --base-url https://api.anthropic.com
ccm list --type api
ccm delete --type password
```

**New syntax (v0.9.1+)**:
```bash
ccm add claude sk-ant-xxx --env ANTHROPIC_API_KEY=SECRET --env ANTHROPIC_BASE_URL=https://api.anthropic.com
ccm list
ccm delete claude
```

### Automatic Migration

When you first run v0.9.1, it will automatically migrate your database:
1. Detect old schema (with `type` column)
2. Convert entries to new format (env var mappings)
3. Remove `type` column and index
4. Preserve all existing data

## Next Steps

1. Read the full documentation: `README.md`
2. Check testing guide: `docs/TESTING.md`
3. Contribute! See source code in `src/`

## Support

- Issues: Check the original project repository
- Documentation: See `README.md` and inline code comments
- Security: For security vulnerabilities, use private disclosure
