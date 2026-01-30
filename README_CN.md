# CCM - 自定义配置管理器

跨平台 CLI 工具，用于管理 API 配置、密码、SSH 密钥和通用密钥，采用军事级加密。

## 特性

**统一条目模型**：所有条目直接存储环境变量映射。无预定义类型 - 完全可定制。

**核心特性**：
- 单一灵活的条目类型，支持自定义元数据
- 使用 `SECRET` 占位符的环境变量映射
- 简洁的 CLI：`ccm add <名称> <密钥> --env 变量=值`
- 跨平台操作系统密钥链集成

## 架构概览

### 统一条目模型

所有条目遵循相同结构：
- **name**：条目标识符
- **metadata**：环境变量映射（例如 `{"API_KEY": "SECRET", "BASE_URL": "https://..."}`）
- **tags**：可选标签，用于组织
- **notes**：可选备注
- **timestamps**：创建和更新时间

### 安全架构

1. **密钥层**：所有密钥在存储前使用 AES-256-GCM 加密
2. **主密钥保护**：PIN 派生密钥（PBKDF2-SHA256，200,000 次迭代）或 ZERO_KEY
3. **操作系统密钥链**：主密钥存储在操作系统密钥链中（Windows DPAPI、macOS Keychain、Linux libsecret）
4. **数据库**：纯 SQLite 用于元数据存储（密钥已预加密）

### 核心模块

- `types/` - 统一条目类型定义
- `core/` - 统一初始化层
- `db/` - 数据库操作（纯 SQLite）
- `secrets/` - 密钥 CRUD 操作和主密钥管理
- `auth/` - 身份验证和 PIN 管理
- `env/` - 环境变量管理（平台特定）
- `commands/` - CLI 命令实现
- `utils/` - 加密工具和验证

## 快速开始

### 方式 1：自动化测试（Windows PowerShell）
```powershell
cd ccm_rust
.\tests\scripts\test.ps1
```

或使用批处理脚本：
```cmd
cd ccm_rust
tests\scripts\test.bat
```

### 方式 2：手动构建

#### 前置要求

- Rust 1.70 或更高版本
- Windows：无额外依赖
- macOS：无额外依赖
- Linux：需要 `libsecret` 支持密钥环（`sudo apt-get install libsecret-1-dev`）

#### 构建命令

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 本地安装
cargo install --path .
```

## 使用方法

### 基础命令

```bash
# 显示版本
ccm version

# 显示帮助
ccm help

# 初始化并设置 PIN
ccm auth set

# 列出所有条目
ccm list
ccm list --json
ccm list --verbose
```

### 添加条目

统一模型使用环境变量映射，`SECRET` 作为占位符：

```bash
# 添加带环境变量的 API 配置
ccm add claude-api "sk-ant-xxx" \
  --env ANTHROPIC_API_KEY=SECRET \
  --env ANTHROPIC_BASE_URL=https://api.anthropic.com

# 使用默认环境变量（从名称派生）
ccm add my-password "hunter2" \
  --env MY_PASSWORD=SECRET

# 添加多个环境变量
ccm add my-service "token123" \
  --env API_KEY=SECRET \
  --env BASE_URL=https://api.example.com \
  --env TIMEOUT=30 \
  --tags production,api
```

### 操作条目

```bash
# 获取条目详情
ccm get claude-api

# 复制密钥到剪贴板
ccm get claude-api -c

# 使用条目（设置环境变量）
ccm use claude-api
# 根据条目元数据设置 ANTHROPIC_API_KEY、ANTHROPIC_BASE_URL

# 搜索条目
ccm search claude

# 更新条目
ccm update claude-api \
  --env ANTHROPIC_API_KEY=SECRET \
  --notes "生产环境 API 密钥"

# 删除条目
ccm delete claude-api
ccm delete entry1 entry2 entry3
```

### 导入和导出

```bash
# 从 CSV 或 JSON 导入
ccm import passwords.csv
ccm import backup.json

# 导出为加密备份
ccm export

# 导出特定条目
ccm export claude-api

# 明文导出（谨慎使用！）
ccm export -d
```

## 环境变量映射

`SECRET` 占位符用于指示哪个环境变量应接收解密的密钥值：

```bash
# 当你运行：
ccm add my-app "my-secret-key" \
  --env APP_API_KEY=SECRET \
  --env APP_BASE_URL=https://api.example.com \
  --env APP_TIMEOUT=30

# 条目存储：
# - APP_API_KEY → "SECRET"（占位符）
# - APP_BASE_URL → "https://api.example.com"（字面值）
# - APP_TIMEOUT → "30"（字面值）

# 当你运行：ccm use my-app
# 它设置：
# - APP_API_KEY = "my-secret-key"（解密的密钥）
# - APP_BASE_URL = "https://api.example.com"
# - APP_TIMEOUT = 30
```

## 常见模式

### API 密钥

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

### 密码

```bash
# 单个密码
ccm add github "my-pass" \
  --env GITHUB_TOKEN=SECRET \
  --notes "个人 GitHub token"

# 带额外元数据
ccm add work-vpn "vpn-secret" \
  --env VPN_PASSWORD=SECRET \
  --env VPN_SERVER=vpn.company.com \
  --tags work,vpn
```

### 配置管理

```bash
# 数据库配置
ccm add prod-db "db-password-123" \
  --env DB_HOST=prod-db.example.com \
  --env DB_PASSWORD=SECRET \
  --env DB_PORT=5432 \
  --env DB_NAME=production \
  --tags production,database
```

## 平台特定功能

### 环境变量

**Windows**：
- 使用 `setx` 设置用户级环境变量
- 存储在注册表中
- 需要新的 shell 会话才能生效

**Unix/macOS**：
- 将 export 语句追加到 shell 配置文件
- 支持：`~/.zshrc`、`~/.bashrc`、`~/.config/fish/config.fish`
- 运行 `source ~/.zshrc` 或重启 shell 使更改生效

### 操作系统密钥链集成

**Windows**：DPAPI（数据保护 API）

**macOS**：Keychain Services

**Linux**：libsecret（gnome-keyring）

## 安全注意事项

### PIN 丢失 = 数据丢失

没有忘记 PIN 的恢复机制。这是为了最大安全性而设计的。

### 基于会话的身份验证

身份验证绑定到你的 shell 进程。会话在 shell 退出时自动过期。

### 主密钥安全

- 32 字节随机主密钥
- PBKDF2-SHA256，200,000 次迭代用于 PIN 派生
- 主密钥仅在会话期间缓存在内存中
- 销毁时内存清零

### 密钥加密

- 所有密钥使用 AES-256-GCM 加密
- 密钥在数据库存储前加密
- 元数据中的 `SECRET` 占位符指示加密值位置

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_encrypt_decrypt
```

### 调试模式

```bash
# 启用调试日志
DEBUG=1 cargo run -- <command>
```

### 代码结构

```
src/
├── main.rs              # 入口点
├── commands/            # CLI 命令实现
├── core/                # 初始化层
├── db/                  # 数据库操作
├── secrets/             # 密钥管理
├── auth/                # 身份验证
├── env/                 # 环境变量
├── types/               # 统一条目类型
└── utils/               # 工具（加密、验证、错误）
```

## 限制

### 平台限制

- 必须有可用的操作系统密钥链
- 交叉编译需要平台特定的构建

## 许可证

MIT

## 贡献

欢迎贡献！请确保：

1. 所有测试通过：`cargo test`
2. 代码已格式化：`cargo fmt`
3. 无 clippy 警告：`cargo clippy`
4. 文档已更新

## 安全披露

对于安全漏洞，请通过 GitHub issues 私下报告。

## 致谢

- Rust 社区提供的优秀加密库
