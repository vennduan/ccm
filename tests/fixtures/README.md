# Test Fixtures Directory

This directory contains test data files for testing CCM's CSV import and JSON import/export functionality.

## Directory Structure

```
fixtures/
├── csv/                    # CSV test files for import testing
│   ├── edge-format.csv
│   ├── chrome-format.csv
│   ├── firefox-format.csv
│   ├── safari-format.csv
│   ├── generic-format.csv
│   ├── special-characters.csv
│   ├── unicode-utf8.csv
│   ├── edge-cases.csv
│   └── windows-crlf.csv
└── json/                   # JSON test files for import/export testing
    ├── complete-backup.json
    ├── minimal-backup.json
    ├── special-characters.json
    ├── encrypted-backup-template.json
    ├── edge-cases.json
    └── all-entry-types.json
```

## CSV Test Files

### Browser Format Files

- **edge-format.csv** - Microsoft Edge password export format
  - Required columns: `name`, `url`, `username`, `password`
  - Optional columns: `note`
  - Example entries: GitHub, Google, Amazon, Netflix, Twitter

- **chrome-format.csv** - Google Chrome password export format
  - Required columns: `url`, `username`, `password`
  - Optional columns: `note`, `group`
  - `group` column maps to entry name in CCM
  - Example entries: GitHub, Google, OpenAI API, AWS

- **firefox-format.csv** - Firefox password export format
  - Required columns: `url`, `username`, `password`
  - Optional columns: `httprealm`, `formactionorigin`, `guid`
  - Example entries: GitHub with realms, Google, API endpoint

- **safari-format.csv** - Safari password export format
  - Required columns: `Username`, `Password`, `URL` (case-sensitive)
  - No optional columns
  - Example entries: GitHub, Google, Amazon

- **generic-format.csv** - Generic CSV format with password column
  - Required columns: `password`
  - All other columns are optional and stored as metadata
  - Example entries: Various services with custom fields

### Edge Case Files

- **special-characters.csv** - Tests special character handling
  - Quoted fields with commas
  - Escaped quotes (`""`)
  - Emoji characters
  - Newlines within quoted fields
  - Special symbols: `!@#$%^&*()_+-=[]{}|;':",./<>?`

- **unicode-utf8.csv** - Tests Unicode and multi-language support
  - Chinese (中文)
  - Japanese (日本語)
  - French (Français)
  - German (Deutsch)
  - Korean (한국어)
  - Russian (Русский)
  - Arabic (العربية)
  - Emoji

- **edge-cases.csv** - Tests various edge cases
  - Empty password fields
  - Empty username fields
  - Missing URL
  - Very long passwords
  - Names with spaces (should be sanitized)
  - Names with special characters (should be sanitized)
  - Uppercase names (should be lowercased)
  - Multiple consecutive hyphens
  - Leading/trailing underscores

- **windows-crlf.csv** - Tests Windows line endings
  - Uses `\r\n` (CRLF) line endings instead of `\n` (LF)
  - Verifies cross-platform compatibility

## JSON Test Files

### Backup Files

- **complete-backup.json** - Full backup with all entry types
  - 5 entries covering all types: `api`, `password`, `ssh`, `secret`
  - Complete metadata for each type
  - Tags and notes included
  - Timestamps included
  - Examples: Claude API, GitHub account, SSH key, OpenAI API, database token

- **minimal-backup.json** - Minimal backup (required fields only)
  - 2 entries with only required fields
  - No optional fields (tags, notes, timestamps)
  - Tests import with missing optional data

- **all-entry-types.json** - Comprehensive example of all entry types
  - API entries: Claude, OpenAI, Gemini, GitHub, Custom
  - Password entries: GitHub, AWS, Netflix
  - SSH entries: Production server, Staging server, GitHub
  - Secret entries: Database URL, API token, License key
  - Complete metadata for each type
  - Multiple tags per entry
  - Notes and timestamps

### Special Character Files

- **special-characters.json** - Tests special characters in JSON
  - Special symbols in passwords: `!@#$%^&*()_+-=[]{}|;':",./<>?`
  - Unicode characters (Chinese, Japanese, etc.)
  - Emoji in notes and secrets
  - Multiline notes with `\n` characters
  - Quotes in notes (`"` and `'` and `` ` ``)

### Edge Case Files

- **edge-cases.json** - Tests various edge cases
  - Very long entry names (should be truncated to 64 chars)
  - SSH entries with all metadata fields
  - API with custom template
  - Password with extensive metadata including custom fields
  - Empty metadata object
  - Missing timestamps (should be auto-generated)

- **encrypted-backup-template.json** - Template for encrypted backups
  - Shows the structure of encrypted backups
  - Contains placeholder for encrypted data
  - Actual encrypted backups are created by the `export` command

## Usage Examples

### Testing CSV Import

```bash
# Import Edge format CSV
ccm import tests/fixtures/csv/edge-format.csv

# Import Chrome format CSV
ccm import tests/fixtures/csv/chrome-format.csv

# Import Safari format CSV
ccm import tests/fixtures/csv/safari-format.csv

# Import generic CSV
ccm import tests/fixtures/csv/generic-format.csv

# Import with special characters
ccm import tests/fixtures/csv/special-characters.csv

# Import Unicode/UTF-8 CSV
ccm import tests/fixtures/csv/unicode-utf8.csv
```

### Testing JSON Import

```bash
# Import complete backup (will prompt for password if encrypted)
ccm import tests/fixtures/json/complete-backup.json

# Import minimal backup
ccm import tests/fixtures/json/minimal-backup.json

# Import special characters JSON
ccm import tests/fixtures/json/special-characters.json

# Import all entry types
ccm import tests/fixtures/json/all-entry-types.json
```

### Testing JSON Export

```bash
# Export all entries (encrypted)
ccm export

# Export all entries (unencrypted, for testing)
ccm export --decrypt

# Export specific type
ccm export --type password

# Export specific entry
ccm export --name github-account
```

## Test Coverage

These test files cover:

### CSV Tests
- ✅ All browser formats (Edge, Chrome, Firefox, Safari, Generic)
- ✅ Special characters and escape sequences
- ✅ Unicode and multi-language support
- ✅ Empty and missing fields
- ✅ Field trimming and whitespace handling
- ✅ Different line endings (LF vs CRLF)
- ✅ Very long fields
- ✅ Quoted and unquoted fields
- ✅ Entry name sanitization

### JSON Tests
- ✅ All entry types (api, password, ssh, secret)
- ✅ Required and optional fields
- ✅ Timestamps (present and missing)
- ✅ Tags and notes
- ✅ Special characters in secrets and metadata
- ✅ Unicode and emoji
- ✅ Empty metadata objects
- ✅ Custom metadata fields
- ✅ Very long entry names
- ✅ Multiline text fields

## Adding New Test Files

When adding new test files:

1. **CSV files**: Ensure they use proper CSV escaping (quotes for fields with commas/newlines)
2. **JSON files**: Ensure valid JSON syntax (use a linter or validator)
3. **File naming**: Use descriptive names (e.g., `special-characters.csv`, `all-entry-types.json`)
4. **Documentation**: Update this README with the new file's purpose

## Notes

- All passwords and secrets in these test files are **dummy values** for testing only
- Do not use these test files with real credentials
- Encrypted JSON backups are created by the `export` command with a user-provided password
- The `encrypted-backup-template.json` shows the structure but contains placeholder data
