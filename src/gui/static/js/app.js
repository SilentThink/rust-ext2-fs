// 全局变量
let currentPath = '';
let commandHistory = [];
let historyIndex = -1;

// DOM元素
const fileListElement = document.getElementById('file-list');
const currentPathElement = document.getElementById('current-path');
const terminalContentElement = document.getElementById('terminal-content');
const terminalInputElement = document.getElementById('terminal-input');
const promptElement = document.getElementById('prompt');

// 当前查看的文件名
let currentViewingFile = '';

// 右键菜单和对话框元素
const contextMenu = document.getElementById('context-menu');
const fileContextMenu = document.getElementById('file-context-menu');
const createFileModal = document.getElementById('create-file-modal');
const createFolderModal = document.getElementById('create-folder-modal');
const createShortcutModal = document.getElementById('create-shortcut-modal');
const fileContentModal = document.getElementById('file-content-modal');
const writeFileModal = document.getElementById('write-file-modal');
const fileNameInput = document.getElementById('file-name');
const folderNameInput = document.getElementById('folder-name');
const shortcutTargetInput = document.getElementById('shortcut-target');
const shortcutNameInput = document.getElementById('shortcut-name');
const fileContentDisplay = document.getElementById('file-content-display');
const fileContentTitle = document.getElementById('file-content-title');
const writeFileNameInput = document.getElementById('write-file-name');
const writeFileContentInput = document.getElementById('write-file-content');
const writeFileBtn = document.getElementById('write-file-btn');

// 当前右键点击的文件信息
let currentRightClickedItem = null;

// 设置终端最大显示行数
const MAX_TERMINAL_LINES = 100;

// 初始化
document.addEventListener('DOMContentLoaded', () => {
    // 获取当前目录内容
    refreshDirectory();
    
    // 设置终端输入事件
    terminalInputElement.addEventListener('keydown', handleTerminalInput);
    
    // 初始化终端欢迎信息
    appendToTerminal('欢迎使用Ext2文件系统可视化界面！', 'system');
    appendToTerminal('输入 "help" 或 "?" 获取可用命令列表。', 'system');
    
    // 设置右键菜单事件
    setupContextMenu();
    
    // 设置对话框事件
    setupModals();
    
    // 设置写入文件按钮事件
    writeFileBtn.addEventListener('click', () => {
        const fileName = writeFileNameInput.value.trim();
        const content = writeFileContentInput.value;
        
        if (fileName) {
            // 调用write命令，传递文件名和内容
            fetch('/api/command', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    cmd: 'write',
                    args: [fileName, content]
                })
            })
            .then(response => response.json())
            .then(data => {
                appendToTerminal(data.output, data.success ? 'output' : 'error');
                if (data.success) {
                    hideModal(writeFileModal);
                    refreshDirectory();
                }
            })
            .catch(error => {
                console.error('写入文件失败:', error);
                appendToTerminal(`写入文件失败: ${error.message}`, 'error');
            });
        } else {
            appendToTerminal('请指定文件名', 'error');
        }
    });
});

// 设置右键菜单
function setupContextMenu() {
    // 为整个文件浏览器区域添加右键菜单
    const fileExplorer = document.querySelector('.file-explorer');
    
    fileExplorer.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        console.log('文件浏览器区域右键点击被触发');
        
        // 如果点击的是文件项，不显示菜单
        if (e.target.closest('.file-item')) {
            console.log('点击了文件项，不显示菜单');
            return;
        }
        
        // 获取鼠标位置
        const x = e.clientX || e.pageX;
        const y = e.clientY || e.pageY;
        
        console.log(`右键菜单位置: x=${x}, y=${y}`);
        
        // 显示菜单并定位
        contextMenu.style.display = 'block';
        contextMenu.style.left = `${x}px`;
        contextMenu.style.top = `${y}px`;
    });
    
    // 阻止右键菜单自身的右键事件
    contextMenu.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        e.stopPropagation();
    });
    
    // 点击其他区域隐藏菜单
    document.addEventListener('mousedown', (e) => {
        // 如果点击的不是菜单区域，隐藏菜单
        if (!contextMenu.contains(e.target) && !fileContextMenu.contains(e.target)) {
            contextMenu.style.display = 'none';
            fileContextMenu.style.display = 'none';
        }
    });
    
    // 绑定菜单项点击事件
    document.getElementById('create-file').addEventListener('click', (e) => {
        console.log('点击新建文件');
        e.stopPropagation(); // 阻止事件冒泡
        
        // 显示创建文件对话框
        showModal(createFileModal);
        fileNameInput.value = '';
        fileNameInput.focus();
        // 隐藏右键菜单
        contextMenu.style.display = 'none';
    });
    
    document.getElementById('create-folder').addEventListener('click', (e) => {
        console.log('点击新建文件夹');
        e.stopPropagation(); // 阻止事件冒泡
        
        // 显示创建文件夹对话框
        showModal(createFolderModal);
        folderNameInput.value = '';
        folderNameInput.focus();
        // 隐藏右键菜单
        contextMenu.style.display = 'none';
    });
    
    // 绑定创建快捷方式菜单项点击事件
    document.getElementById('create-shortcut').addEventListener('click', (e) => {
        console.log('点击创建快捷方式');
        e.stopPropagation();
        
        if (currentRightClickedItem) {
            // 显示创建快捷方式对话框
            shortcutTargetInput.value = currentRightClickedItem.name;
            shortcutNameInput.value = currentRightClickedItem.name + '_shortcut';
            showModal(createShortcutModal);
            shortcutNameInput.focus();
        }
        
        // 隐藏右键菜单
        fileContextMenu.style.display = 'none';
    });
    
    // 按下ESC键隐藏菜单
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            contextMenu.style.display = 'none';
            fileContextMenu.style.display = 'none';
        }
    });
}

// 设置对话框
function setupModals() {
    // 关闭按钮事件
    document.querySelectorAll('.modal-close, .modal-close-btn').forEach(button => {
        button.addEventListener('click', (e) => {
            const modal = e.target.closest('.modal');
            if (modal) {
                hideModal(modal);
            }
        });
    });
    
    // ESC键关闭对话框
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            document.querySelectorAll('.modal').forEach(modal => {
                hideModal(modal);
            });
        }
    });
    
    // 创建文件按钮事件
    document.getElementById('create-file-btn').addEventListener('click', () => {
        const fileName = fileNameInput.value.trim();
        if (fileName) {
            // 执行touch命令创建文件
            executeCommand('touch', [fileName]);
            hideModal(createFileModal);
            fileNameInput.value = '';
        }
    });
    
    // 创建文件夹按钮事件
    document.getElementById('create-folder-btn').addEventListener('click', () => {
        const folderName = folderNameInput.value.trim();
        if (folderName) {
            // 执行mkdir命令创建文件夹
            executeCommand('mkdir', [folderName]);
            hideModal(createFolderModal);
            folderNameInput.value = '';
        }
    });
    
    // 回车键确认创建
    fileNameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            document.getElementById('create-file-btn').click();
        }
    });
    
    folderNameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            document.getElementById('create-folder-btn').click();
        }
    });
    
    // 写入文件对话框回车键支持
    writeFileNameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            writeFileContentInput.focus();
        }
    });
    
    writeFileContentInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && e.ctrlKey) {
            writeFileBtn.click();
            e.preventDefault();
        }
    });
    
    // 编辑文件按钮事件
    document.getElementById('edit-file-btn').addEventListener('click', () => {
        if (currentViewingFile) {
            // 隐藏文件内容查看对话框
            hideModal(fileContentModal);
            // 显示写入文件对话框进行编辑
            showWriteFileDialog(currentViewingFile);
        }
    });
    
    // 创建快捷方式按钮事件
    document.getElementById('create-shortcut-btn').addEventListener('click', () => {
        const target = shortcutTargetInput.value.trim();
        const shortcutName = shortcutNameInput.value.trim();
        
        if (target && shortcutName) {
            // 执行ln -s命令创建软链接
            executeCommand('ln', ['-s', target, shortcutName]);
            hideModal(createShortcutModal);
            shortcutTargetInput.value = '';
            shortcutNameInput.value = '';
        }
    });
    
    // 快捷方式名称输入框回车键支持
    shortcutNameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            document.getElementById('create-shortcut-btn').click();
        }
    });
}

// 显示对话框
function showModal(modal) {
    modal.style.display = 'flex';
}

// 隐藏对话框
function hideModal(modal) {
    modal.style.display = 'none';
}

// 刷新目录内容
async function refreshDirectory() {
    try {
        const response = await fetch('/api/directory');
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        currentPath = data.path;
        
        // 更新路径导航
        updatePathNavigator(currentPath);
        
        // 更新终端提示符
        updatePrompt(currentPath);
        
        // 清空文件列表
        fileListElement.innerHTML = '';
        
        // 不需要手动添加返回上级目录选项，因为后端已经提供了这个选项
        
        // 添加文件和目录
        data.items.forEach(item => {
            const fileItem = createFileItem(item);
            fileListElement.appendChild(fileItem);
        });

        // 滚动到文件列表顶部
        fileListElement.scrollTop = 0;
    } catch (error) {
        console.error('获取目录内容失败:', error);
        appendToTerminal(`获取目录内容失败: ${error.message}`, 'error');
    }
}

// 创建文件项元素
function createFileItem(item) {
    const fileItem = document.createElement('div');
    fileItem.className = 'file-item';
    
    const icon = document.createElement('span');
    // 根据文件类型设置图标样式
    if (item.is_symlink) {
        if (item.is_dir) {
            icon.className = 'file-icon symlink-dir';
            icon.innerHTML = '<i class="fas fa-folder"></i>';
        } else {
            icon.className = 'file-icon symlink';
            icon.innerHTML = '<i class="fas fa-file"></i>';
        }
    } else {
        icon.className = `file-icon ${item.is_dir ? 'folder' : 'file'}`;
        icon.innerHTML = item.is_dir ? '<i class="fas fa-folder"></i>' : '<i class="fas fa-file"></i>';
    }
    
    const name = document.createElement('span');
    name.className = 'file-name';
    name.textContent = item.name;
    
    const details = document.createElement('div');
    details.className = 'file-details';
    
    if (item.size) {
        const size = document.createElement('span');
        size.textContent = item.size;
        details.appendChild(size);
    }
    
    if (item.owner) {
        const owner = document.createElement('span');
        owner.textContent = item.owner;
        details.appendChild(owner);
    }
    
    fileItem.appendChild(icon);
    fileItem.appendChild(name);
    fileItem.appendChild(details);
    
    // 添加右键菜单事件
    fileItem.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        e.stopPropagation();
        
        // 跳过返回上级目录项
        if (item.name === '..') {
            return;
        }
        
        // 保存当前右键点击的文件信息
        currentRightClickedItem = item;
        
        // 获取鼠标位置
        const x = e.clientX || e.pageX;
        const y = e.clientY || e.pageY;
        
        // 显示文件右键菜单
        fileContextMenu.style.display = 'block';
        fileContextMenu.style.left = `${x}px`;
        fileContextMenu.style.top = `${y}px`;
        
        // 隐藏普通右键菜单
        contextMenu.style.display = 'none';
    });
    
    // 添加双击事件
    fileItem.addEventListener('dblclick', () => {
        if (item.is_symlink) {
            // 如果是软链接，根据目标类型处理
            if (item.is_dir) {
                // 软链接指向目录，执行cd命令
                executeCommand('cd', [item.name]);
            } else {
                // 软链接指向文件，显示文件内容
                showFileContent(item.name);
            }
        } else if (item.is_dir) {
            // 如果是目录，执行cd命令
            const dirName = item.name === '..' ? '..' : item.name;
            executeCommand('cd', [dirName]);
        } else {
            // 如果是文件，显示文件内容
            showFileContent(item.name);
        }
    });
    
    return fileItem;
}

// 更新路径导航
function updatePathNavigator(path) {
    currentPathElement.textContent = path;
}

// 更新终端提示符
function updatePrompt(path) {
    promptElement.textContent = `[${path}]$ `;
}

// 处理终端输入
function handleTerminalInput(event) {
    if (event.key === 'Enter') {
        const input = terminalInputElement.value.trim();
        if (input) {
            // 添加到命令历史
            commandHistory.push(input);
            historyIndex = commandHistory.length;
            
            // 显示输入的命令
            appendToTerminal(`${promptElement.textContent}${input}`, 'input');
            
            // 解析命令
            const parts = input.split(' ');
            const cmd = parts[0];
            const args = parts.slice(1).filter(arg => arg.trim() !== '');
            
            // 执行命令
            executeCommand(cmd, args);
            
            // 清空输入框
            terminalInputElement.value = '';
        }
    } else if (event.key === 'ArrowUp') {
        // 上一条命令
        if (historyIndex > 0) {
            historyIndex--;
            terminalInputElement.value = commandHistory[historyIndex];
        }
        event.preventDefault();
    } else if (event.key === 'ArrowDown') {
        // 下一条命令
        if (historyIndex < commandHistory.length - 1) {
            historyIndex++;
            terminalInputElement.value = commandHistory[historyIndex];
        } else {
            historyIndex = commandHistory.length;
            terminalInputElement.value = '';
        }
        event.preventDefault();
    }
}

// 执行命令
async function executeCommand(cmd, args) {
    try {
        // 处理清屏命令
        if (cmd === 'clear') {
            clearTerminal();
            return;
        }
        
        // 特殊处理write命令，显示写入文件对话框
        if (cmd === 'write') {
            showWriteFileDialog(args[0] || '');
            return;
        }
        
        // 特殊处理cd命令，因为它会改变当前目录
        if (cmd === 'cd') {
            const response = await fetch('/api/cd', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(args[0] || '')
            });
            
            const data = await response.json();
            if (data.success) {
                // 刷新目录内容
                refreshDirectory();
                // 自动执行pwd命令显示当前路径
                executeCommand('pwd', []);
            } else {
                appendToTerminal(data.output, 'error');
            }
        } else {
            // 其他命令
            const response = await fetch('/api/command', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    cmd,
                    args
                })
            });
            
            const data = await response.json();
            appendToTerminal(data.output, data.success ? 'output' : 'error');
            
            // 如果命令可能改变了文件系统状态，刷新目录内容
            if (['mkdir', 'touch', 'rm', 'rmdir', 'cp', 'write', 'ln'].includes(cmd)) {
                refreshDirectory();
            }
        }
    } catch (error) {
        console.error('执行命令失败:', error);
        appendToTerminal(`执行命令失败: ${error.message}`, 'error');
    }
}

// 添加内容到终端
function appendToTerminal(text, type) {
    // 先检查是否需要清理旧内容
    limitTerminalLines(true);
    
    const line = document.createElement('div');
    line.className = `terminal-line ${type}`;
    line.textContent = text;
    
    terminalContentElement.appendChild(line);
    
    // 确保滚动到底部
    scrollTerminalToBottom();
}

// 限制终端行数
function limitTerminalLines(preCleanup = false) {
    const lines = terminalContentElement.children;
    // 如果是预清理，或者行数超过限制，进行清理
    if (preCleanup && lines.length >= MAX_TERMINAL_LINES || lines.length > MAX_TERMINAL_LINES) {
        // 计算需要移除的行数
        const removeCount = lines.length - MAX_TERMINAL_LINES + (preCleanup ? 1 : 0);
        
        // 一次性移除多个元素
        if (removeCount > 0) {
            for (let i = 0; i < removeCount; i++) {
                if (lines.length > 0) {
                    terminalContentElement.removeChild(lines[0]);
                }
            }
        }
    }
}

// 清除终端内容
function clearTerminal() {
    terminalContentElement.innerHTML = '';
}

// 添加滚动到底部的函数
function scrollTerminalToBottom() {
    terminalContentElement.scrollTop = terminalContentElement.scrollHeight;
}

// 显示文件内容
async function showFileContent(fileName) {
    try {
        // 保存当前查看的文件名
        currentViewingFile = fileName;
        
        // 设置对话框标题
        fileContentTitle.textContent = `文件内容: ${fileName}`;
        
        // 清空内容显示区域
        fileContentDisplay.textContent = '加载中...';
        
        // 显示对话框
        showModal(fileContentModal);
        
        // 执行cat命令获取文件内容
        const response = await fetch('/api/command', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                cmd: 'cat',
                args: [fileName]
            })
        });
        
        const data = await response.json();
        
        if (data.success) {
            // 显示文件内容
            fileContentDisplay.textContent = data.output;
        } else {
            // 显示错误信息
            fileContentDisplay.textContent = `无法读取文件: ${data.output}`;
        }
    } catch (error) {
        console.error('获取文件内容失败:', error);
        fileContentDisplay.textContent = `获取文件内容失败: ${error.message}`;
    }
}

// 显示写入文件对话框
function showWriteFileDialog(fileName = '') {
    // 设置对话框标题和文件名
    writeFileNameInput.value = fileName;
    writeFileContentInput.value = '';
    
    // 如果指定了文件名且文件已存在，尝试读取内容
    if (fileName) {
        fetch('/api/command', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                cmd: 'cat',
                args: [fileName]
            })
        })
        .then(response => response.json())
        .then(data => {
            if (data.success) {
                writeFileContentInput.value = data.output;
            }
        })
        .catch(error => {
            console.error('读取文件内容失败:', error);
        });
    }
    
    // 显示对话框
    showModal(writeFileModal);
    
    // 聚焦到文件名输入框，如果没有文件名
    if (!fileName) {
        writeFileNameInput.focus();
    } else {
        writeFileContentInput.focus();
    }
}