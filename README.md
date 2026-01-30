# CCM - Custom Configuration Manager

A cross-platform CLI tool for managing API configurations, passwords, SSH keys, and generic secrets with military-grade encryption.

## Features

**Unified Entry Model**: All entries store environment variable mappings directly. No predefined types - completely customizable for your needs.

**Key Features**:
- Single flexible entry type with custom metadata
- Environment variable mappings with `SECRET` placeholder
- Simple CLI: `ccm add <name> <secret> --env VAR=VALUE`
- Cross-platform OS keychain integration

## Architecture Overview

### Unified Entry Model

All entries now follow the same structure:
- **name**: Entry identifier
- **metadata**: Environment variable mappings (e.g., `{"API_KEY": "SECRET", "BASE_URL": "https://..."}`)
- **tags**: Optional tags for organization
- **notes**: Optional notes
- **timestamps**: Creation and update times

### Security Architecture

1. **Secret Layer**: AES-256-GCM encryption for all secrets before storage
2. **Master Key Protection**: PIN-derived key (PBKDF2-SHA256, 200,000 iterations) or ZERO_KEY
3. **OS Keychain**: Master key stored in OS keychain (Windows DPAPI, macOS Keychain, Linux libsecret)
4. **Database**: Plain SQLite for metadata storage (secrets are pre-encrypted)

### Core Modules

- `types/` - Unified entry type definitions
- `core/` - Unified initialization layer
- `db/` - Database operations (plain SQLite)
- `secrets/` - Secret CRUD operations and master key management
- `auth/` - Authentication and PIN management
- `env/` - Environment variable management (platform-specific)
- `commands/` - CLI command implementations
- `utils/` - Cryptographic utilities and validation

## Quick Start

### Option 1: Automated Test (Windows PowerShell)
```powershell
cd ccm_rust
.\tests\scripts\test.ps1
```

Or use batch script:
```cmd
cd ccm_rust
tests\scripts\test.bat
```

### Option 2: Manual Build

#### Prerequisites

- Rust 1.70 or later
- Windows: No additional dependencies
- macOS: No additional dependencies
- Linux: `libsecret` for keyring support (`sudo apt-get install libsecret-1-dev`)

#### Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Usage

### Basic Commands

```bash
# Show version
ccm version

# Show help
ccm help

# Initialize and set PIN
ccm auth set

# List all entries
ccm list
ccm list --json
ccm list --verbose
```

### Adding Entries

The new unified model uses environment variable mappings with `SECRET` as placeholder:

```bash
# Add an API configuration with env vars
ccm add claude-api "sk-ant-xxx" \
  --env ANTHROPIC_API_KEY=SECRET \
  --env ANTHROPIC_BASE_URL=https://api.anthropic.com

# Add with default env var (derived from name)
ccm add my-password "hunter2" \
  --env MY_PASSWORD=SECRET

# Add multiple env vars
ccm add my-service "token123" \
  --env API_KEY=SECRET \
  --env BASE_URL=https://api.example.com \
  --env TIMEOUT=30 \
  --tags production,api
```

### Working with Entries

```bash
# Get entry details
ccm get claude-api

# Copy secret to clipboard
ccm get claude-api -c

# Use entry (set environment variables)
ccm use claude-api
# Sets ANTHROPIC_API_KEY, ANTHROPIC_BASE_URL based on entry metadata

# Search entries
ccm search claude

# Update an entry
ccm update claude-api \
  --env ANTHROPIC_API_KEY=SECRET \
  --notes "Production API key"

# Delete entries
ccm delete claude-api
ccm delete entry1 entry2 entry3
```

### Import and Export

```bash
# Import from CSV or JSON
ccm import passwords.csv
ccm import backup.json

# Export to encrypted backup
ccm export

# Export specific entry
ccm export claude-api

# Plaintext export (use with caution!)
ccm export -d
```

## Environment Variable Mappings

The `SECRET` placeholder is used to indicate which environment variable should receive the decrypted secret value:

```bash
# When you run this:
ccm add my-app "my-secret-key" \
  --env APP_API_KEY=SECRET \
  --env APP_BASE_URL=https://api.example.com \
  --env APP_TIMEOUT=30

# The entry stores:
# - APP_API_KEY → "SECRET" (placeholder)
# - APP_BASE_URL → "https://api.example.com" (literal value)
# - APP_TIMEOUT → "30" (literal value)

# When you run: ccm use my-app
# It sets:
# - APP_API_KEY = "my-secret-key" (decrypted secret)
# - APP_BASE_URL = "https://api.example.com"
# - APP_TIMEOUT = 30
```

## Common Patterns

### API Keys

```bash
# Claude API
ccm add claude "sk-ant-xxx" \
  --env ANTHROPIC_API_KEY=SECRET \
  --env ANTHROPIC_BASE_URL=https://api.anthropic.com \
  --env ANTHROPIC_MODEL=claude-sonnet-4-20250514

# OpenAI API
ccm add openai "sk-xxx" \
  --env OPENAI_API_KEY=SECRET \
  --env OPENAI_BASE_URL=https://api.openai.com \
  --env OPENAI_MODEL=gpt-4
```

### Passwords

```bash
# Single password
ccm add github "my-pass" \
  --env GITHUB_TOKEN=SECRET \
  --notes "Personal GitHub token"

# With additional metadata
ccm add work-vpn "vpn-secret" \
  --env VPN_PASSWORD=SECRET \
  --env VPN_SERVER=vpn.company.com \
  --tags work,vpn
```

### Configuration Management

```bash
# Database configuration
ccm add prod-db "db-password-123" \
  --env DB_HOST=prod-db.example.com \
  --env DB_PASSWORD=SECRET \
  --env DB_PORT=5432 \
  --env DB_NAME=production \
  --tags production,database
```

## Platform-Specific Features

### Environment Variables

**Windows**:
- Uses `setx` for user-level environment variables
- Stored in registry
- Requires new shell session for changes to take effect

**Unix/macOS**:
- Appends export statements to shell config files
- Supports: `~/.zshrc`, `~/.bashrc`, `~/.config/fish/config.fish`
- Run `source ~/.zshrc` or restart shell for changes

### OS Keychain Integration

**Windows**: DPAPI (Data Protection API)

**macOS**: Keychain Services

**Linux**: libsecret (gnome-keyring)

## Security Considerations

### PIN Loss = Data Loss

There is no recovery mechanism for forgotten PINs. This is by design for maximum security.

### Session-Based Authentication

Authentication is tied to your shell process. The session automatically expires when the shell exits.

### Master Key Security

- 32-byte random master key
- PBKDF2-SHA256 with 200,000 iterations for PIN derivation
- Master key cached in memory only during session
- Memory zeroization on drop

### Secret Encryption

- AES-256-GCM encryption for all secrets
- Secrets encrypted before database storage
- `SECRET` placeholder in metadata indicates encrypted value location

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_encrypt_decrypt
```

### Debug Mode

```bash
# Enable debug logging
DEBUG=1 cargo run -- <command>
```

### Code Structure

```
src/
├── main.rs              # Entry point
├── commands/            # CLI command implementations
├── core/                # Initialization layer
├── db/                  # Database operations
├── secrets/             # Secret management
├── auth/                # Authentication
├── env/                 # Environment variables
├── types/               # Unified entry type
└── utils/               # Utilities (crypto, validation, errors)
```

## Limitations

### Platform Limitations

- OS keychain must be available
- Cross-compilation requires platform-specific builds

## License

MIT

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated

## Security Disclosure

For security vulnerabilities, please report them privately via GitHub issues.

## Acknowledgments

- Rust community for excellent cryptographic libraries
