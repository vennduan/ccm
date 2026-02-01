# Windows 平台使用 vcpkg 安装 OpenSSL 静态库并编译 SQLCipher

## 概述

本文档说明如何在 Windows 平台上通过 vcpkg 安装 OpenSSL 静态库，并将其静态链接到 SQLCipher 中，实现单文件分发。

---

## 一、前置要求

### 1. Visual Studio 构建工具

需要安装 Visual Studio 的 C++ 构建工具：

- Visual Studio 2019/2022（Community 版本即可）
- 或者单独安装 "Build Tools for Visual Studio"
- 必须包含 "Desktop development with C++" 工作负载

### 2. Git

用于克隆 vcpkg 和 SQLCipher 仓库。

---

## 二、安装 vcpkg

### 1. 克隆 vcpkg 仓库

```powershell
# 选择一个安装目录（例如 D:\Dev）
cd D:\Dev
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
```

### 2. 初始化 vcpkg

```powershell
.\bootstrap-vcpkg.bat
```

### 3. 集成到 Visual Studio（可选）

```powershell
# 需要管理员权限
.\vcpkg.exe integrate install
```

---

## 三、安装 OpenSSL 静态库

### 1. 理解 vcpkg triplet

vcpkg 使用 "triplet" 来描述目标平台和链接方式：

| Triplet | 说明 |
|---------|------|
| `x64-windows` | 动态链接库（DLL），动态链接 MSVC 运行时 |
| `x64-windows-static` | 静态库，**静态链接** MSVC 运行时（/MT） |
| `x64-windows-static-md` | 静态库，**动态链接** MSVC 运行时（/MD）⭐ **推荐** |

**关键区别**：
- `static`: 库本身是静态的 + MSVC 运行时也静态链接（`/MT`）
- `static-md`: 库本身是静态的 + MSVC 运行时动态链接（`/MD`）

**推荐使用 `x64-windows-static-md`**：
- 符合 Microsoft 官方推荐的运行时分发方式
- 避免多个静态运行时副本导致的问题
- 最终用户仍然只需要一个 `.exe` 文件

### 2. 安装命令

```powershell
# 64 位（推荐）
.\vcpkg.exe install openssl:x64-windows-static-md

# 或者使用完全静态（包括运行时）
.\vcpkg.exe install openssl:x64-windows-static

# 32 位
.\vcpkg.exe install openssl:x86-windows-static-md
```

### 3. 安装位置

安装完成后，文件位于：

```
vcpkg\packages\openssl_x64-windows-static-md\
├── include\
│   └── openssl\
│       ├── aes.h
│       ├── ssl.h
│       └── ...
└── lib\
    ├── libcrypto.lib  (静态库)
    └── libssl.lib     (静态库)
```

---

## 四、编译 SQLCipher

### 1. 克隆 SQLCipher

```powershell
cd D:\Dev
git clone https://github.com/sqlcipher/sqlcipher.git
cd sqlcipher
```

### 2. 复制 OpenSSL 库（简化路径）

```powershell
# 将 vcpkg 安装的 OpenSSL 复制到 SQLCipher 目录
cp -r ..\vcpkg\packages\openssl_x64-windows-static-md .\
```

### 3. 修改 Makefile.msc

打开 `Makefile.msc`，找到以下部分：

```makefile
# Flags controlling use of the in memory btree implementation
#
# SQLITE_TEMP_STORE is 0 to force temporary tables to be in a file, 1 to
# default to file, 2 to default to memory, and 3 to force temporary
# tables to always be in memory.
#
TCC = $(TCC) -DSQLITE_TEMP_STORE=1
RCC = $(RCC) -DSQLITE_TEMP_STORE=1
```

**替换为**：

```makefile
# Flags controlling use of the in memory btree implementation
#
# SQLITE_TEMP_STORE is 0 to force temporary tables to be in a file, 1 to
# default to file, 2 to default to memory, and 3 to force temporary
# tables to always be in memory.
#
TCC = $(TCC) -DSQLITE_TEMP_STORE=2
RCC = $(RCC) -DSQLITE_TEMP_STORE=2

# Enable SQLCipher encryption
TCC = $(TCC) -DSQLITE_HAS_CODEC
RCC = $(RCC) -DSQLITE_HAS_CODEC

# Include OpenSSL headers
!IF "$(PLATFORM)"=="x64"
TCC = $(TCC) -I"openssl_x64-windows-static-md\include"
RCC = $(RCC) -I"openssl_x64-windows-static-md\include"
!ELSEIF "$(PLATFORM)"=="x86"
TCC = $(TCC) -I"openssl_x86-windows-static-md\include"
RCC = $(RCC) -I"openssl_x86-windows-static-md\include"
!ENDIF

# Link OpenSSL static libraries
!IF "$(PLATFORM)"=="x64"
LTLIBPATHS = $(LTLIBPATHS) /LIBPATH:"openssl_x64-windows-static-md\lib"
LTLIBS = $(LTLIBS) libcrypto.lib libssl.lib
!ELSEIF "$(PLATFORM)"=="x86"
LTLIBPATHS = $(LTLIBPATHS) /LIBPATH:"openssl_x86-windows-static-md\lib"
LTLIBS = $(LTLIBS) libcrypto.lib libssl.lib
!ENDIF

# OpenSSL dependencies (Windows system libraries)
LTLIBS = $(LTLIBS) WS2_32.Lib Gdi32.Lib AdvAPI32.Lib Crypt32.Lib User32.Lib
```

**关键修改说明**：
1. `SQLITE_TEMP_STORE=2`: SQLCipher 要求临时表存储在内存中
2. `SQLITE_HAS_CODEC`: 启用加密支持
3. `-I"openssl_x64-windows-static-md\include"`: 指定 OpenSSL 头文件路径
4. `/LIBPATH:"openssl_x64-windows-static-md\lib"`: 指定 OpenSSL 库文件路径
5. `libcrypto.lib libssl.lib`: 链接 OpenSSL 静态库
6. `WS2_32.Lib ...`: OpenSSL 依赖的 Windows 系统库

### 4. 编译

打开 **Visual Studio Native Tools Command Prompt**：

- 64 位：`x64 Native Tools Command Prompt for VS 2022`
- 32 位：`x86 Native Tools Command Prompt for VS 2022`

```cmd
cd D:\Dev\sqlcipher
nmake /f Makefile.msc
```

### 5. 编译产物

编译成功后会生成：

```
sqlcipher\
├── libsqlite3.lib   (静态库，包含 OpenSSL)
├── sqlite3.dll      (动态库)
├── sqlite3.exe      (命令行工具)
└── ...
```

**重命名建议**：
```powershell
# 大多数工具期望 SQLCipher 使用 "sqlcipher" 前缀
ren libsqlite3.lib libsqlcipher.lib
ren sqlite3.dll sqlcipher.dll
```

---

## 五、CCM 项目的配置方式（推荐）

CCM 项目采用**项目内静态库**的方式，避免依赖外部路径：

### 1. 复制文件到项目目录

编译完成后，将文件复制到 CCM 项目根目录：

```powershell
# 假设你的 SQLCipher 编译目录是 D:\Dev\sqlcipher
# CCM 项目目录是 M:\Projects\Tools\CCManager\ccm_rust

# 复制库文件到项目 lib/ 目录
New-Item -ItemType Directory -Path "M:\Projects\Tools\CCManager\ccm_rust\lib" -Force
Copy-Item "D:\Dev\sqlcipher\sqlite3_static.lib" "M:\Projects\Tools\CCManager\ccm_rust\lib\"
Copy-Item "D:\Dev\vcpkg\packages\openssl_x64-windows-static-md\lib\libcrypto.lib" "M:\Projects\Tools\CCManager\ccm_rust\lib\"
Copy-Item "D:\Dev\vcpkg\packages\openssl_x64-windows-static-md\lib\libssl.lib" "M:\Projects\Tools\CCManager\ccm_rust\lib\"

# 复制头文件到项目 include/ 目录
New-Item -ItemType Directory -Path "M:\Projects\Tools\CCManager\ccm_rust\include" -Force
Copy-Item "D:\Dev\sqlcipher\sqlite3.h" "M:\Projects\Tools\CCManager\ccm_rust\include\"
Copy-Item "D:\Dev\sqlcipher\sqlite3ext.h" "M:\Projects\Tools\CCManager\ccm_rust\include\"
Copy-Item -Recurse "D:\Dev\vcpkg\packages\openssl_x64-windows-static-md\include\openssl" "M:\Projects\Tools\CCManager\ccm_rust\include\"
```

### 2. 项目目录结构

配置完成后，CCM 项目目录结构：

```
ccm_rust/
├── lib/
│   ├── sqlite3_static.lib    (SQLCipher 静态库)
│   ├── libcrypto.lib         (OpenSSL 加密库)
│   └── libssl.lib            (OpenSSL SSL 库)
├── include/
│   ├── sqlite3.h             (SQLite 头文件)
│   ├── sqlite3ext.h          (SQLite 扩展头文件)
│   └── openssl/              (OpenSSL 头文件目录)
│       ├── aes.h
│       ├── evp.h
│       └── ...
├── .cargo/
│   └── config.toml           (配置使用项目内库)
├── build.rs                  (链接脚本)
└── Cargo.toml
```

### 3. 配置文件

`.cargo/config.toml` (需要从模板创建)：

```bash
# 复制模板
cp .cargo/config.toml.example .cargo/config.toml

# 编辑 config.toml，修改路径为你的项目绝对路径
```

示例配置：
```toml
[env]
OPENSSL_NO_VENDOR = "1"
OPENSSL_STATIC = "1"
OPENSSL_DIR = "C:/path/to/ccm_rust"                    # ← 修改为你的路径
OPENSSL_LIB_DIR = "C:/path/to/ccm_rust/lib"            # ← 修改为你的路径
OPENSSL_INCLUDE_DIR = "C:/path/to/ccm_rust/include"    # ← 修改为你的路径
```

**注意**：
- `config.toml` 已被 gitignore，每个开发者需要自己创建
- 必须使用绝对路径（相对路径在 build scripts 中不工作）
- Linux/WSL 用户应删除此文件，使用系统 OpenSSL

`build.rs` (已配置，无需修改)：
```rust
#[cfg(all(windows, target_env = "msvc"))]
fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={}/lib", manifest_dir);
    println!("cargo:rustc-link-lib=static=libcrypto");
    println!("cargo:rustc-link-lib=static=libssl");
    // ... Windows 系统库
}
```

### 4. 编译项目

```powershell
cd M:\Projects\Tools\CCManager\ccm_rust
cargo build --release
```

---

## 六、通用 Rust 项目配置（参考）

如果你的项目不是 CCM，可以参考以下配置：

### 1. Cargo.toml 配置

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled-sqlcipher"] }
```

### 2. 环境变量配置

```powershell
# 设置 SQLCipher 库路径
$env:SQLCIPHER_LIB_DIR = "D:\Dev\sqlcipher"
$env:SQLCIPHER_INCLUDE_DIR = "D:\Dev\sqlcipher"

# 构建项目
cargo build --release
```

### 3. build.rs 脚本（如果需要）

创建 `build.rs`：

```rust
#[cfg(all(windows, feature = "bundled-sqlcipher"))]
fn main() {
    // 链接 OpenSSL 静态库（SQLCipher 依赖）
    println!("cargo:rustc-link-lib=static=libcrypto");
    println!("cargo:rustc-link-lib=static=libssl");

    // 链接 Windows 系统库（OpenSSL 依赖）
    println!("cargo:rustc-link-lib=ws2_32");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=crypt32");
    println!("cargo:rustc-link-lib=user32");
}

#[cfg(not(all(windows, feature = "bundled-sqlcipher")))]
fn main() {}
```

---

## 六、验证静态链接

### 1. 检查依赖项

使用 [Dependencies](https://github.com/lucasg/Dependencies) 工具：

```powershell
# 下载 Dependencies.exe
# 打开你的 .exe 文件
```

**静态链接成功的标志**：
- ❌ 不应该看到 `libcrypto-*.dll`
- ❌ 不应该看到 `libssl-*.dll`
- ✅ 只有 Windows 系统 DLL（kernel32.dll, user32.dll 等）

### 2. 文件大小对比

| 链接方式 | 可执行文件大小 | 需要分发的文件 |
|---------|--------------|--------------|
| 动态链接 | ~2 MB | .exe + libcrypto.dll + libssl.dll |
| 静态链接 | ~5-8 MB | 仅 .exe |

---

## 七、常见问题

### 1. 找不到 Visual Studio

**错误**：
```
Error: in triplet x64-windows: Unable to find a valid Visual Studio instance
```

**解决**：
- 安装 Visual Studio 2019/2022 的 C++ 工作负载
- 或安装 "Build Tools for Visual Studio"

### 2. OpenSSL 链接错误

**错误**：
```
unresolved external symbol EVP_EncryptInit_ex
```

**解决**：
- 确保 `build.rs` 中链接了 `libcrypto` 和 `libssl`
- 确保链接了 Windows 系统库（ws2_32, crypt32 等）

### 3. 运行时缺少 VCRUNTIME140.dll

**原因**：使用了 `x64-windows-static` 而不是 `x64-windows-static-md`

**解决**：
- 重新安装 OpenSSL：`vcpkg install openssl:x64-windows-static-md`
- 或者分发 Visual C++ Redistributable

---

## 八、与 CCM 当前方案对比

| 方案 | 构建复杂度 | 分发大小 | 用户体验 | 安全性 |
|------|-----------|---------|---------|--------|
| **bundled SQLite + 应用层加密** | ⭐⭐⭐⭐⭐ 简单 | 基准 | 单文件 | 军事级 |
| **bundled-sqlcipher (vcpkg)** | ⭐⭐⭐ 中等 | +3-5 MB | 单文件 | 军事级 + 数据库加密 |
| **动态链接 SQLCipher** | ⭐⭐ 复杂 | 基准 | 需要 DLL | 军事级 + 数据库加密 |

**CCM 当前选择的合理性**：
- ✅ 构建简单（无需 vcpkg + Visual Studio）
- ✅ 跨平台一致（Rust 自带工具链）
- ✅ 安全性相同（应用层 AES-256-GCM）
- ✅ 分发简单（单文件）

**如果需要数据库级加密**：
- 使用 vcpkg 方案可以实现
- 增加 3-5 MB 文件大小
- 需要维护 OpenSSL 版本更新

---

## 九、参考资料

1. [vcpkg 官方文档](https://github.com/microsoft/vcpkg)
2. [Statically Linking SQLCipher on Windows](https://blog.hamaluik.ca/posts/statically-linking-sqlcipher-on-windows/)
3. [Compiling SQLCipher on Windows and Linux](https://gufeng.sh/note/compiling-sqlcipher/)
4. [Microsoft Learn - vcpkg Triplets](https://learn.microsoft.com/en-us/vcpkg/users/platforms/windows)
5. [SQLCipher Community Edition](https://www.zetetic.net/sqlcipher/introduction)

---

## 十、总结

通过 vcpkg 安装 OpenSSL 静态库并编译 SQLCipher 的完整流程：

1. **安装 vcpkg** → 克隆仓库 + bootstrap
2. **安装 OpenSSL** → `vcpkg install openssl:x64-windows-static-md`
3. **修改 Makefile** → 添加 OpenSSL 路径和链接选项
4. **编译 SQLCipher** → `nmake /f Makefile.msc`
5. **集成到 Rust** → 配置环境变量或 build.rs

**最终结果**：单个 `.exe` 文件，包含 SQLCipher + OpenSSL，无需额外 DLL。

**适用场景**：
- 需要数据库级加密（防止直接读取数据库文件）
- 可以接受 3-5 MB 的文件大小增加
- 有 Visual Studio 构建环境

**不适用场景**：
- 追求最小文件大小
- 应用层加密已经足够（如 CCM）
- 希望避免复杂的构建依赖
