#!/bin/bash
# Test SQLCipher encryption

set -e

echo "=== SQLCipher Encryption Test ==="
echo

# Clean up old database
rm -rf ~/.ccm
echo "✓ Cleaned up old database"

# Initialize and set PIN
echo "Setting PIN..."
echo -e "1234\n1234" | ./target/release/ccm auth set
echo "✓ PIN set"

# Add a test entry
echo
echo "Adding test entry..."
./target/release/ccm add test-key "my-secret-value" --env TEST_KEY=SECRET
echo "✓ Entry added"

# Verify entry exists
echo
echo "Verifying entry..."
./target/release/ccm get test-key
echo "✓ Entry retrieved"

# Test database encryption
echo
echo "=== Testing Database Encryption ==="
DB_PATH=~/.ccm/ccm.db

if [ ! -f "$DB_PATH" ]; then
    echo "❌ Database file not found"
    exit 1
fi

echo "Database file: $DB_PATH"
echo "File size: $(du -h $DB_PATH | cut -f1)"

# Try to open with regular SQLite (should fail)
echo
echo "Attempting to open with regular SQLite..."
if sqlite3 "$DB_PATH" "SELECT * FROM entries;" 2>&1 | grep -q "file is not a database\|file is encrypted"; then
    echo "✅ SUCCESS: Database is encrypted! Regular SQLite cannot read it."
else
    echo "❌ FAIL: Database is NOT encrypted. Regular SQLite can read it."
    echo
    echo "Database content:"
    sqlite3 "$DB_PATH" "SELECT name, substr(metadata, 1, 50) FROM entries;"
    exit 1
fi

# Check if SQLCipher is available
if command -v sqlcipher &> /dev/null; then
    echo
    echo "Testing with SQLCipher..."
    # Try without key (should fail)
    if sqlcipher "$DB_PATH" "SELECT * FROM entries;" 2>&1 | grep -q "file is not a database\|file is encrypted"; then
        echo "✅ SQLCipher without key: Cannot read (expected)"
    fi
else
    echo
    echo "ℹ️  sqlcipher command not found (optional)"
fi

echo
echo "=== Test Summary ==="
echo "✅ Database file is encrypted with SQLCipher"
echo "✅ Regular SQLite cannot read the database"
echo "✅ Application can read/write encrypted data"
echo
echo "🎉 All tests passed!"
