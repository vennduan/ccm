# CCM 加密/解密流程文档

**版本**: 0.9.1
**平台**: Rust 实现
**更新日期**: 2025-01-31

---

## 一、加密架构概述

CCM 使用**双层加密**架构：

```
┌─────────────────────────────────────────────────────────────┐
│                        用户数据                               │
│                    (API Keys, 密码等)                         │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              第一层：Secret 加密 (AES-256-GCM)                │
│                   每条 secret 单独加密                         │
│            密钥来源：Master Key (32字节随机)                  │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│           第二层：数据库加密 (SQLCipher AES-256-CBC)          │
│            整个数据库文件加密 (Unix 平台)                      │
│            密钥来源：Master Key 派生 (hex 编码)                │
│            Windows: 标准 SQLite (应用层加密已足够)            │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  第三层：Master Key 保护                     │
│              存储在 OS Keychain (DPAPI/Keychain/libsecret)   │
│            保护方式：PIN 派生密钥 OR ZERO_KEY                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、Master Key 生命周期

### 2.1 Master Key 生成

```rust
// 位置: src/secrets/master_key.rs
pub fn generate_master_key() -> [u8; 32] {
    // 使用加密安全的随机数生成器
    use rand::Rng;
    let mut key = [0u8; 32];
    let mut rng = rand::thread_rng();
    rng.fill(&key);
    key
}
```

**特点**：
- 32 字节 (256 位)
- 使用 `rand::thread_rng()` 生成
- 符合军事级加密标准

### 2.2 Master Key 存储

```
Master Key (32 bytes)
    │
    ├── 压缩 (gzip)
    │
    ├── 加密 (AES-256-GCM)
    │   ├── 密钥: PIN 派生密钥 OR ZERO_KEY
    │   ├── IV: 12 字节随机
    │   └── Auth Tag: 16 字节
    │
    └── 序列化为 JSON
        {
          "iv": "base64(iv)",
          "ciphertext": "base64(加密后的密钥)",
          "authTag": "base64(认证标签)"
        }
        │
        └── 存储到 OS Keychain
            服务名: "ccm-{instance_id}"
            条目名: "master-key"
```

### 2.3 Master Key 加载流程

```
┌──────────────────────────────────────────────────────────────┐
│                    get_cached_master_key()                   │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────┐
              │  检查内存缓存               │
              │  MASTER_KEY_CACHE          │
              └────────────┬───────────────┘
                           │
                 ┌─────────┴─────────┐
                 │                   │
            已缓存               未缓存
                 │                   │
                 ▼                   ▼
            返回密钥      ┌────────────────────────┐
                         │ 检查是否设置了 PIN      │
                         │ pin::has_pin()         │
                         │ (使用 OS Keychain)     │
                         └───────────┬────────────┘
                                     │
                           ┌─────────┴─────────┐
                           │                   │
                      有 PIN               无 PIN
                           │                   │
                           ▼                   ▼
                  返回 PinRequired    load_master_key()
                                       │
                                       ▼
                              ┌─────────────────────┐
                              │ load_master_key()   │
                              │ 读取 OS Keychain    │
                              │ 使用 ZERO_KEY 解密   │
                              └──────────┬──────────┘
                                         │
                               ┌─────────┴─────────┐
                               │                   │
                          找到密钥             未找到
                               │                   │
                               ▼                   ▼
                          缓存并返回    generate_and_save_master_key()
                                                 │
                                                 ▼
                                          生成新密钥
                                          保存到 Keychain
                                          缓存并返回
```

---

## 三、PIN 管理

### 3.1 PIN 设置流程 (`ccm auth set`)

```
┌──────────────────────────────────────────────────────────────┐
│                     auth set 命令                             │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
            ┌──────────────────────────────┐
            │ 检查是否已设置 PIN            │
            │ pin::has_pin()               │
            └────────────┬─────────────────┘
                         │
               ┌─────────┴─────────┐
               │                   │
          已设置                未设置
               │                   │
               ▼                   ▼
          返回错误        1. 生成随机 Salt (32 字节)
                       2. 使用 PBKDF2-SHA256 (200,000 次迭代) 哈希 PIN
                       3. 存储到数据库 settings 表:
                          - pinHash: 哈希后的 PIN (hex)
                          - pinSalt: Salt (hex)
                       4. 在 OS Keychain 设置 PIN 标志
                       5. 加载 Master Key (使用 ZERO_KEY)
                       6. 使用新 PIN 重新加密 Master Key
                       7. 更新 OS Keychain
```

### 3.2 PIN 验证流程

```
┌──────────────────────────────────────────────────────────────┐
│                    pin::verify_pin(pin)                      │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
            ┌──────────────────────────────┐
            │ 从数据库读取 pinSalt          │
            │ db.get_setting("pinSalt")    │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 使用相同 Salt 和迭代次数      │
            │ 哈希输入的 PIN                │
            │ PBKDF2-SHA256, 200,000 次    │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 使用 constant-time 比较哈希值 │
            │ 防止时序攻击                  │
            └────────────┬─────────────────┘
                         │
               ┌─────────┴─────────┐
               │                   │
          匹配                不匹配
               │                   │
               ▼                   ▼
          返回 true          返回 false
```

---

## 四、Secret 加密/解密流程

### 4.1 添加 Secret (`ccm add`)

```
用户输入: ccm add my-api --secret sk-123456
    │
    ▼
┌───────────────────────────────────────────────────────────────┐
│ 1. 创建 Entry (元数据)                                         │
│    metadata = {"SECRET": "SECRET", "BASE_URL": "...", ...}   │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 2. 获取 Master Key                                            │
│    get_cached_master_key()                                    │
│    - 检查缓存                                                  │
│    - 如需要，从 Keychain 加载                                  │
│    - 如需要，生成新密钥                                        │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 3. AES-256-GCM 加密 Secret 值                                 │
│    encrypt_aes256_gcm(master_key, secret_value.as_bytes())    │
│    - 生成随机 IV (12 字节)                                    │
│    - 加密数据                                                 │
│    - 生成 Auth Tag (16 字节)                                  │
│    - 返回: IV + 密文 + Auth Tag                               │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 4. Hex 编码加密结果                                            │
│    hex::encode(encrypted_data)                                │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 5. 存储到数据库                                               │
│    entries 表: 元数据 (metadata, tags, notes, etc.)          │
│    secrets 表: 加密后的 secret (hex 编码)                      │
└───────────────────────────────────────────────────────────────┘
```

### 4.2 获取 Secret (`ccm get`)

```
用户输入: ccm get my-api
    │
    ▼
┌───────────────────────────────────────────────────────────────┐
│ 1. 从数据库读取 Entry 和加密的 Secret                          │
│    db.get_entry(name)                                         │
│    db.get_secret(name)                                        │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 2. 获取 Master Key                                            │
│    get_cached_master_key()                                    │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 3. Hex 解密密文                                               │
│    hex::decode(encrypted_hex)                                 │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 4. AES-256-GCM 解密                                           │
│    decrypt_aes256_gcm(master_key, encrypted_bytes)           │
│    - 解析 IV 和 Auth Tag                                      │
│    - 验证并解密                                               │
│    - 返回原始字节                                             │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────────────────┐
│ 5. 转换为字符串并返回                                          │
│    String::from_utf8(decrypted_bytes)                         │
└───────────────────────────────────────────────────────────────┘
```

---

## 五、数据库加密流程

### 5.1 数据库初始化

```
┌───────────────────────────────────────────────────────────────┐
│                     Database::new()                           │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
            ┌──────────────────────────────┐
            │ 获取 Master Key              │
            │ get_cached_master_key()      │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 派生数据库密钥                │
            │ db_key = hex::encode(master_key) │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 打开/创建数据库               │
            │ Connection::open(path)       │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 设置加密密钥 (SQLCipher)      │
            │ PRAGMA key = "{db_key}"      │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 配置加密参数                  │
            │ - cipher: aes-256-cbc        │
            │ - kdf_iter: 256000           │
            │ - page_size: 4096            │
            │ - journal_mode: WAL          │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 创建/验证表结构               │
            │ - entries 表                 │
            │ - secrets 表                 │
            │ - settings 表                │
            └──────────────────────────────┘
```

### 5.2 平台差异

| 平台 | 数据库加密 | Secret 加密 | 说明 |
|------|-----------|-------------|------|
| Windows | ✅ SQLCipher (AES-256-CBC) | ✅ AES-256-GCM | 双层保护 |
| macOS | ✅ SQLCipher (AES-256-CBC) | ✅ AES-256-GCM | 双层保护 |
| Linux | ✅ SQLCipher (AES-256-CBC) | ✅ AES-256-GCM | 双层保护 |

---

## 六、初始化和认证流程

### 6.1 系统初始化

```
┌───────────────────────────────────────────────────────────────┐
│                    main() 启动                               │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           ▼
            ┌──────────────────────────────┐
            │ core::initialize()          │
            │ (只执行一次，结果缓存)        │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 1. 检查 OS Secret Service    │
            │    check_os_secret_service_… │
            │    - Windows: DPAPI          │
            │    - macOS: Keychain         │
            │    - Linux: libsecret        │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 2. 检查 Master Key 是否存在  │
            │    has_master_key()          │
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 3. 检查是否需要迁移           │
            │    migration::needs_migration()│
            └────────────┬─────────────────┘
                         │
                         ▼
            ┌──────────────────────────────┐
            │ 4. 创建默认配置 (首次运行)    │
            │    migration::should_create_… │
            └──────────────────────────────┘
```

### 6.2 认证流程

```
命令: ccm auth on
    │
    ▼
┌───────────────────────────────────────────────────────────────┐
│ 1. 检查是否已认证                                             │
│    auth::is_authenticated()                                  │
│    - 检查认证状态文件                                         │
│    - 验证 shell 进程是否仍在运行                              │
└──────────────────────────┬────────────────────────────────────┘
                           │
                 ┌─────────┴─────────┐
                 │                   │
             已认证              未认证
                 │                   │
                 ▼                   ▼
            返回       ┌──────────────────────────┐
                       │ 检查是否设置了 PIN        │
                       │ pin::has_pin()           │
                       └──────────┬───────────────┘
                                  │
                        ┌─────────┴─────────┐
                        │                   │
                    有 PIN              无 PIN
                        │                   │
                        ▼                   ▼
              ┌────────────────┐   ┌────────────────┐
              │ 提示输入 PIN    │   │ 加载 Master Key│
              │ verify_pin()   │   │ (ZERO_KEY)     │
              └───────┬────────┘   └───────┬────────┘
                      │                    │
                      ▼                    ▼
              ┌────────────────┐   ┌────────────────┐
              │ 加载 Master Key│   │ 设置认证状态   │
              │ (使用 PIN)     │   │ auth::set_…    │
              └───────┬────────┘   └────────────────┘
                      │
                      ▼
              ┌────────────────┐
              │ 设置认证状态   │
              │ auth::set_…    │
              └────────────────┘
```

---

## 七、关键安全设计

### 7.1 防止循环依赖

**问题**: `Database::new()` 需要 Master Key，但 `get_cached_master_key()` 可能需要访问数据库（读取 instance_id）。

**解决方案**: `get_instance_id_from_config()` 直接使用 `rusqlite::Connection` 打开数据库，不通过 `get_database()`，避免循环。

```rust
// 直接数据库访问，避免循环依赖
pub fn get_instance_id_from_config() -> Result<Option<String>> {
    let db_path = crate::db::db_path();
    if !db_path.exists() {
        return Ok(None);
    }
    let conn = Connection::open(&db_path)?;
    // 直接查询 instance_id
    // ...
}
```

### 7.2 PIN 检查优化

**问题**: 旧的实现中 `has_pin()` 需要访问数据库，导致循环依赖。

**解决方案**: 使用 OS Keychain 存储一个标志 `ccm-pin-set`，避免数据库访问。

```rust
pub fn has_pin() -> Result<bool> {
    let entry = keyring::Entry::new("ccm", "ccm-pin-set")?;
    match entry.get_password() {
        Ok(_) => Ok(true),   // 标志存在 = PIN 已设置
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(CcmError::Unknown(...)),
    }
}
```

### 7.3 时序安全比较

PIN 验证使用常量时间比较，防止时序攻击：

```rust
use subtle::ConstantTimeEq;

// 或使用手动实现的 constant-time 比较
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}
```

### 7.4 内存安全

Master Key 在内存中的缓存使用 `zeroize` 在丢弃时清零：

```rust
impl Drop for MasterKeyCache {
    fn drop(&mut self) {
        if let Some(mut key) = self.key.take() {
            key.zeroize(); // 安全清零
        }
    }
}
```

---

## 八、加密算法参数总结

| 用途 | 算法 | 密钥长度 | IV/Nonce | 其他参数 |
|------|------|---------|----------|----------|
| Secret 加密 | AES-256-GCM | 256 位 | 96 位 (12 字节) | Auth Tag: 128 位 |
| Master Key 存储 | AES-256-GCM | 256 位 | 96 位 | 压缩: gzip |
| PIN 哈希 | PBKDF2-HMAC-SHA256 | 256 位 | Salt: 256 位 | 迭代: 200,000 次 |
| 数据库加密 | SQLCipher AES-256-CBC | 256 位 | - | KDF 迭代: 256,000 次 |

---

## 九、数据存储位置

| 数据 | 位置 | 格式 | 加密 |
|------|------|------|------|
| Master Key | OS Keychain | JSON (加密后) | ✅ PIN/ZERO_KEY |
| PIN 哈希 | SQLite settings 表 | hex | ❌ (哈希值) |
| PIN Salt | SQLite settings 表 | hex | ❌ (公开) |
| Instance ID | SQLite settings 表 | JSON | ❌ (公开) |
| Secrets | SQLite secrets 表 | hex | ✅ AES-256-GCM |
| Entries 元数据 | SQLite entries 表 | JSON | ❌ (公开) |

---

## 十、常见场景流程

### 场景 1: 首次运行

```
用户运行: ccm list
    │
    ├── initialize() - 检查系统状态
    ├── has_master_key() - 返回 false
    ├── migration::should_create_defaults() - 返回 true
    ├── generate_and_save_master_key() - 生成并保存新密钥
    ├── migration::create_default_profiles() - 创建默认配置
    └── list_entries() - 显示空列表
```

### 场景 2: 设置 PIN

```
用户运行: ccm auth set
    │
    ├── 提示输入新 PIN
    ├── 生成随机 Salt (32 字节)
    ├── PBKDF2 哈希 PIN
    ├── 存储到数据库 (pinHash, pinSalt)
    ├── 设置 Keychain 标志 (ccm-pin-set)
    ├── load_master_key_for_session(None) - 加载现有 Master Key
    ├── reencrypt_master_key(None, Some(new_pin)) - 用 PIN 重新加密
    └── 更新 Keychain
```

### 场景 3: 使用 PIN 登录

```
用户运行: ccm auth on
    │
    ├── has_pin() - 返回 true
    ├── 提示输入 PIN
    ├── verify_pin(pin) - 验证 PIN
    ├── load_master_key_for_session(Some(pin))
    │   ├── get_pin_salt() - 从数据库获取 Salt
    │   ├── derive_key_from_pin(pin, salt) - 派生密钥
    │   ├── load_master_key_with_pin(pin) - 解密 Master Key
    │   └── 缓存到内存
    └── set_authenticated(true) - 设置认证状态
```

### 场景 4: 添加 API Key

```
用户运行: ccm add claude --secret sk-123
    │
    ├── require_authenticated() - 检查认证
    ├── get_cached_master_key() - 获取 Master Key
    ├── encrypt_aes256_gcm(master_key, b"sk-123") - 加密
    ├── hex::encode() - 编码
    ├── db.save_entry() - 保存元数据
    └── db.save_secret() - 保存加密的 secret
```

---

## 十一、错误处理

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `OsSecretServiceRequired` | OS Keychain 不可用 | 安装相应的密钥管理服务 |
| `PinRequired` | PIN 已设置但未提供 | 运行 `ccm auth on` 并输入 PIN |
| `InvalidPin` | PIN 验证失败 | 重新输入或使用 `ccm auth change` |
| `MasterKeyNotAvailable` | Master Key 不存在且无法生成 | 运行 `ccm auth set` 初始化 |
| `EntryNotFound` | 条目不存在 | 使用 `ccm list` 查看可用条目 |
| `Encryption("Decryption failed")` | 密钥不匹配或数据损坏 | 检查 Master Key 是否正确加载 |

---

*文档结束*
