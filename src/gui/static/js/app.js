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
});

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
    icon.className = `file-icon ${item.is_dir ? 'folder' : 'file'}`;
    icon.innerHTML = item.is_dir ? '<i class="fas fa-folder"></i>' : '<i class="fas fa-file"></i>';
    
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
    
    // 添加双击事件
    fileItem.addEventListener('dblclick', () => {
        if (item.is_dir) {
            // 如果是目录，执行cd命令
            const dirName = item.name === '..' ? '..' : item.name;
            executeCommand('cd', [dirName]);
        } else {
            // 如果是文件，执行cat命令
            executeCommand('cat', [item.name]);
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
            if (['mkdir', 'touch', 'rm', 'rmdir', 'cp'].includes(cmd)) {
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