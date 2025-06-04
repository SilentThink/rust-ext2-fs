# Tab键自动补全功能测试指南

本文档提供了详细的测试步骤来验证rust-ext2-fs中的tab键自动补全功能。

## 🚀 快速启动测试

1. **启动程序**：
   ```bash
   cd rust-ext2-fs
   cargo run
   ```

2. **登录系统**：
   ```
   username: root
   password: 123
   ```

## 📋 分步测试清单

### 阶段1：基础命令补全测试

#### ✅ 测试1.1：单字母命令补全
```bash
[/root] l<Tab>           # 预期结果：显示 "ln  login  ls"
[/root] c<Tab>           # 预期结果：显示 "cat  cd  chmod  chown  cp  clear"
[/root] r<Tab>           # 预期结果：显示 "rm  rmdir"
```

#### ✅ 测试1.2：精确命令补全
```bash
[/root] hi<Tab>          # 预期结果：直接补全为 "history "
[/root] who<Tab>         # 预期结果：直接补全为 "whoami "
[/root] pwd<Tab>         # 预期结果：直接补全为 "pwd "
```

#### ✅ 测试1.3：部分匹配命令补全
```bash
[/root] use<Tab>         # 预期结果：显示 "useradd  userdel"
[/root] ch<Tab>          # 预期结果：显示 "chmod  chown"
```

### 阶段2：文件系统准备

在进行文件补全测试前，先创建测试环境：

```bash
[/root] mkdir documents photos temp workspace
[/root] touch file1.txt file2.txt readme.md config.json
[/root] touch documents/report.doc documents/notes.txt
[/root] mkdir documents/projects documents/archives
[/root] touch photos/image1.jpg photos/image2.png
[/root] mkdir workspace/src workspace/tests
[/root] touch workspace/src/main.rs workspace/src/lib.rs
```

### 阶段3：文件路径补全测试

#### ✅ 测试3.1：文件名补全
```bash
[/root] cat f<Tab>       # 预期结果：显示 "file1.txt  file2.txt"
[/root] cat file1<Tab>   # 预期结果：直接补全为 "cat file1.txt "
[/root] rm readme<Tab>   # 预期结果：直接补全为 "rm readme.md "
[/root] cat config<Tab>  # 预期结果：直接补全为 "cat config.json "
```

#### ✅ 测试3.2：目录补全（带斜杠）
```bash
[/root] cd d<Tab>        # 预期结果：直接补全为 "cd documents/"
[/root] ls p<Tab>        # 预期结果：直接补全为 "ls photos/"
[/root] cd w<Tab>        # 预期结果：直接补全为 "cd workspace/"
```

#### ✅ 测试3.3：子目录补全
```bash
[/root] cd documents/<Tab>    # 预期结果：显示 "projects/  archives/"
[/root] ls documents/p<Tab>   # 预期结果：直接补全为 "ls documents/projects/"
[/root] cat documents/r<Tab>  # 预期结果：直接补全为 "cat documents/report.doc "
```

### 阶段4：高级功能测试

#### ✅ 测试4.1：多次Tab键循环
```bash
[/root] touch test1.txt test2.txt test3.txt
[/root] cat test<Tab>         # 第1次：显示所有候选项
[/root] cat test<Tab><Tab>    # 第2次：选择 test1.txt
[/root] cat test<Tab><Tab><Tab> # 第3次：选择 test2.txt
```

#### ✅ 测试4.2：空目录测试
```bash
[/root] cd temp/
[/root/temp] ls <Tab>         # 预期结果：无响应（空目录）
[/root/temp] cat <Tab>        # 预期结果：无响应（无文件）
```

#### ✅ 测试4.3：无匹配项测试
```bash
[/root] cat xyz<Tab>          # 预期结果：无响应（无匹配文件）
[/root] zzz<Tab>              # 预期结果：无响应（无匹配命令）
```

### 阶段5：综合场景测试

#### ✅ 测试5.1：复杂文件操作
```bash
[/root] cp documents/notes.txt w<Tab>    # 补全为workspace/
[/root] mv workspace/src/main.rs d<Tab>  # 补全为documents/
[/root] ln -s photos/image1.jpg i<Tab>   # 测试软链接创建
```

#### ✅ 测试5.2：命令组合补全
```bash
[/root] zip documents/report.doc arch<Tab>  # 测试zip命令的文件补全
[/root] chmod rwx:r-- f<Tab>                # 测试chmod命令的文件补全
[/root] chown alice d<Tab>                  # 测试chown命令的目录补全
```

## 🐛 已知限制和注意事项

1. **文件名包含空格**：当前实现对包含空格的文件名支持有限
2. **特殊字符**：某些特殊字符可能影响补全效果
3. **大文件目录**：目录中文件过多时补全响应可能较慢
4. **权限限制**：无读权限的目录无法进行补全

## ✅ 测试通过标准

- [ ] 所有命令补全正常工作
- [ ] 文件路径补全准确显示
- [ ] 目录补全自动添加斜杠
- [ ] 多候选项正确显示
- [ ] 循环选择功能正常
- [ ] 空目录和无匹配情况处理得当
- [ ] 复杂路径补全工作正常

## 🔧 故障排除

如果遇到问题：

1. **补全无响应**：
   - 检查是否在正确的目录
   - 确认文件/目录确实存在
   - 验证当前用户权限

2. **候选项显示错误**：
   - 重新启动程序
   - 检查文件系统状态
   - 确认输入的前缀正确

3. **程序崩溃**：
   - 查看错误信息
   - 报告bug并提供复现步骤

## 📊 性能测试

可选的性能测试：

```bash
# 创建大量文件测试补全性能
[/root] mkdir perf_test
[/root] cd perf_test/
# 创建100个文件（可以通过脚本实现）
[/root/perf_test] touch file{001..100}.txt
[/root/perf_test] cat f<Tab>  # 测试大量候选项的响应时间
```

完成所有测试后，tab键自动补全功能应该能显著提升shell的使用体验！ 