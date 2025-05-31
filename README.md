# rust-ext2-fs

> 多用户 / 多目录文件系统 (Unix)

支持用户添加/删除，文件读写，权限控制等功能。提供终端Shell界面和基于Web的可视化图形界面，方便用户直观操作文件系统。

## 代码结构

```
src
├── fs
│  ├── constant.rs   // 定义了一些文件系统的常量，如块大小、磁盘大小等
│  ├── core          // 核心数据结构，磁盘块的管理
│  │  ├── file.rs    // 处理文件权限
│  │  ├── fs.rs      // 整个文件系统 Fs 的定义，磁盘块的回收/删除
│  │  ├── inode.rs   // inode 结点
│  │  ├── iter.rs    // 实现对 DirEntry 的迭代
│  │  ├── mod.rs    
│  │  ├── traits.rs  // 将 Inode / DirEntry 转换成字节数组的 trait
│  │  └── utils.rs   // 常用函数，如字符串与字节数组的转换
│  ├── func          // 拓展文件系统的功能，提供常用的接口
│  │  ├── chdir.rs   // 更改当前目录
│  │  ├── chmod.rs   // 修改权限
│  │  ├── chown.rs   // 修改文件拥有者
│  │  ├── close.rs   // 关闭文件
│  │  ├── create.rs  // 创建文件 / 创建文件夹
│  │  ├── format.rs  // 格式化文件系统
│  │  ├── init.rs    // 从磁盘初始化文件系统
│  │  ├── login.rs   // 登录 / 切换用户
│  │  ├── mod.rs    
│  │  ├── open.rs    // 打开文件
│  │  ├── passwd.rs  // 修改密码
│  │  ├── path.rs    // 简单的路径解析器，实现通过路径查找文件/文件夹
│  │  ├── pwd.rs     // 查看当前目录的绝对路径
│  │  ├── read.rs    // 读文件
│  │  ├── rm.rs      // 删除文件
│  │  ├── rmdir.rs   // 删除空文件夹
│  │  ├── seek.rs    // 修改文件指针
│  │  ├── useradd.rs // 添加用户
│  │  ├── userdel.rs // 删除用户
│  │  └── write.rs   // 写文件
│  └── mod.rs
├── lib.rs
├── main.rs
├── gui               // 可视化图形界面
│  ├── mod.rs         // Web服务器和API实现
│  └── static         // 静态资源文件
│     ├── css         // 样式文件
│     ├── js          // 前端JavaScript代码
│     └── index.html  // 前端页面入口
├── shell            // 模拟一个 shell，使用文件系统提供的接口
│  ├── cmd           // shell 支持的命令
│  │  ├── cat.rs     // 显示文件内容
│  │  ├── cd.rs      // 修改当前目录
│  │  ├── chmod.rs   // 修改文件权限
│  │  ├── chown.rs   // 修改文件拥有者
│  │  ├── cp.rs      // 复制文件
│  │  ├── exit.rs    // 退出终端
│  │  ├── format.rs  // 格式化
│  │  ├── help.rs    // 显示帮助信息
│  │  ├── login.rs   // 切换用户
│  │  ├── ls.rs      // 显示目录信息
│  │  ├── mkdir.rs   // 创建文件夹
│  │  ├── mod.rs
│  │  ├── passwd.rs  // 修改密码
│  │  ├── pwd.rs     // 查询当前目录
│  │  ├── rm.rs      // 删除文件 / 文件夹
│  │  ├── rmdir.rs   // 删除空文件夹
│  │  ├── touch.rs   // 创建文件
│  │  ├── useradd.rs // 添加用户
│  │  ├── userdel.rs // 删除用户
│  │  ├── users.rs   // 显示用户/密码
│  │  ├── whoami.rs  // 显示当前用户
│  │  └── write.rs   // 写文件
│  └── mod.rs
└── utils.rs
```

## 编译运行

1. 安装 Rust:

  - Linux / Macos

  ```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  
  - Windows

  [安装 Rustup](https://forge.rust-lang.org/infra/other-installation-methods.html#other-ways-to-install-rustup)

2. 切换到这个目录后运行 `cargo run`

   所有依赖会自动安装，主要包括：
   - chrono: 处理时间和日期
   - crossterm: 终端彩色文本显示
   - actix-web: Web服务器框架
   - serde: 序列化/反序列化支持
   - tokio: 异步运行时
   - env_logger: 日志系统

3. 启动可视化界面:

  ```
  cargo run -- --web
  ```

  启动后可通过浏览器访问 http://localhost:8080 使用Web界面

## 可视化界面

该项目提供了基于Web的可视化界面，方便用户直观地操作文件系统。可视化界面具有以下特点：

- 文件资源管理器：直观显示目录结构和文件
- 文件属性展示：显示文件大小、所有者、权限和时间信息
- 集成终端：在Web界面中执行命令
- 响应式设计：适配不同屏幕尺寸

### GUI代码结构

```
src
├── gui
│  ├── mod.rs       // Web服务器实现，处理API请求
│  └── static       // 静态资源
│     ├── css       // 样式文件
│     ├── js        // JavaScript代码
│     └── index.html // Web界面入口
```

### 主要功能

- `/api/directory` - 获取当前目录内容
- `/api/cd` - 更改当前目录
- `/api/command` - 执行Shell命令

### 系统要求

- 支持现代Web标准的浏览器（Chrome, Firefox, Edge, Safari等）
- 8080端口可用

## 开发环境 

Visual Studio Code.

## 文档编译

cargo doc --no-deps --document-private-items --release --open

## Commit 提交规范

为保持代码提交的一致性和可读性，本项目采用以下commit图标分类：

| 图标 | 类型 | 说明 |
|------|------|------|
| ✨ | feat | 新功能，如添加命令、文件系统新特性等 |
| 🐛 | fix | 修复bug，如修复文件读写错误、权限问题等 |
| 📝 | docs | 文档更新，如README、代码注释等 |
| 💄 | style | 代码格式修改，非功能性更改 |
| ♻️ | refactor | 代码重构，既不修复bug也不添加新功能 |
| ⚡️ | perf | 性能优化，如优化文件存取速度、内存使用等 |
| ✅ | test | 添加或修改测试代码 |
| 🔧 | chore | 构建过程或辅助工具的变动 |
| 🧠 | fs | 文件系统核心逻辑相关的更改 |
| 🖥️ | shell | 命令行Shell相关的更改 |
| 🌐 | gui | 图形界面相关的更改 |
| 🔐 | security | 安全相关更新，如用户权限、数据完整性等 |
| 📦 | storage | 存储机制相关的更改，如磁盘块管理、inode等 |

### 提交格式

```
<图标> <类型>(<范围>): <简短描述>
```

- **图标**: 使用上表中的emoji
- **类型**: 对应图标的类型名称
- **范围**: 可选，表示更改影响的范围，如`fs`、`inode`、`dir`等
- **描述**: 简短描述更改内容

### 提交示例

```
✨ feat(shell): 添加cat命令支持显示二进制文件
🐛 fix(fs): 修复大文件写入时的内存泄漏问题
📝 docs: 更新文件系统API文档
💄 style: 统一代码缩进风格
♻️ refactor(core): 重构inode管理逻辑
⚡️ perf(fs): 优化目录项查找算法，提高文件访问速度
✅ test: 添加文件权限测试用例
🔧 chore: 更新Rust依赖版本
🧠 fs: 改进文件系统块分配算法
🖥️ shell: 增强命令行自动补全功能
🌐 gui: 添加文件拖放支持
🔐 security: 增强用户密码存储安全性
📦 storage: 改进磁盘空间回收机制
```
