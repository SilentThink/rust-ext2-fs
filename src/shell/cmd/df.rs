use super::*;
use crate::shell::Shell;
use crate::fs::{BLOCKS, BLOCK_SIZE, DATA_BLOCKS};

pub struct Df;

impl Df {
    /// 格式化字节数为人类可读的格式
    fn format_bytes(bytes: u64, human_readable: bool) -> String {
        if !human_readable {
            return bytes.to_string();
        }

        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{:.0}{}", size, UNITS[unit_index])
        } else {
            format!("{:.1}{}", size, UNITS[unit_index])
        }
    }

    /// 显示文件系统信息
    fn show_filesystem_info(shell: &Shell, human_readable: bool, show_inodes: bool) {
        let fs_desc = shell.fs.fs_desc();
        
        // 计算块使用情况
        let total_blocks = BLOCKS as u64;
        let free_blocks = fs_desc.free_blocks_count as u64;
        let used_blocks = total_blocks - free_blocks;
        
        // 计算字节数
        let block_size = BLOCK_SIZE as u64;
        let total_bytes = total_blocks * block_size;
        let used_bytes = used_blocks * block_size;
        let free_bytes = free_blocks * block_size;
        
        // 计算使用百分比
        let usage_percent = if total_blocks > 0 {
            (used_blocks as f64 / total_blocks as f64 * 100.0) as u32
        } else {
            0
        };

        if show_inodes {
            // 显示inode信息
            let total_inodes = DATA_BLOCKS as u64;
            let free_inodes = fs_desc.free_inodes_count as u64;
            let used_inodes = total_inodes - free_inodes;
            let inode_usage_percent = if total_inodes > 0 {
                (used_inodes as f64 / total_inodes as f64 * 100.0) as u32
            } else {
                0
            };

            println!("{:<15} {:>10} {:>10} {:>10} {:>6} {}", 
                     "Filesystem", "Inodes", "IUsed", "IFree", "IUse%", "Mounted on");
            println!("{:<15} {:>10} {:>10} {:>10} {:>5}% {}", 
                     "ext2fs", 
                     total_inodes,
                     used_inodes,
                     free_inodes,
                     inode_usage_percent,
                     "/");
        } else {
            // 显示块/字节信息
            if human_readable {
                println!("{:<15} {:>10} {:>10} {:>10} {:>6} {}", 
                         "Filesystem", "Size", "Used", "Avail", "Use%", "Mounted on");
                println!("{:<15} {:>10} {:>10} {:>10} {:>5}% {}", 
                         "ext2fs",
                         Self::format_bytes(total_bytes, true),
                         Self::format_bytes(used_bytes, true),
                         Self::format_bytes(free_bytes, true),
                         usage_percent,
                         "/");
            } else {
                println!("{:<15} {:>10} {:>10} {:>10} {:>6} {}", 
                         "Filesystem", "1K-blocks", "Used", "Available", "Use%", "Mounted on");
                println!("{:<15} {:>10} {:>10} {:>10} {:>5}% {}", 
                         "ext2fs",
                         total_bytes / 1024,
                         used_bytes / 1024,
                         free_bytes / 1024,
                         usage_percent,
                         "/");
            }
        }
    }

    /// 显示详细信息
    fn show_detailed_info(shell: &Shell) {
        let fs_desc = shell.fs.fs_desc();
        
        println!("Filesystem Information:");
        println!("======================");
        println!("Volume Name: {}", crate::fs::utils::str(&fs_desc.volume_name));
        println!("Block Size: {} bytes", BLOCK_SIZE);
        println!("Total Blocks: {}", BLOCKS);
        println!("Data Blocks: {}", DATA_BLOCKS);
        println!("Free Blocks: {}", fs_desc.free_blocks_count);
        println!("Used Blocks: {}", BLOCKS - fs_desc.free_blocks_count as usize);
        println!();
        
        let total_bytes = BLOCKS * BLOCK_SIZE;
        let used_bytes = (BLOCKS - fs_desc.free_blocks_count as usize) * BLOCK_SIZE;
        let free_bytes = fs_desc.free_blocks_count as usize * BLOCK_SIZE;
        
        println!("Space Information:");
        println!("-----------------");
        println!("Total Size: {} ({} bytes)", Self::format_bytes(total_bytes as u64, true), total_bytes);
        println!("Used Space: {} ({} bytes)", Self::format_bytes(used_bytes as u64, true), used_bytes);
        println!("Free Space: {} ({} bytes)", Self::format_bytes(free_bytes as u64, true), free_bytes);
        println!();
        
        println!("Inode Information:");
        println!("-----------------");
        println!("Total Inodes: {}", DATA_BLOCKS);
        println!("Free Inodes: {}", fs_desc.free_inodes_count);
        println!("Used Inodes: {}", DATA_BLOCKS - fs_desc.free_inodes_count as usize);
        println!("Directories: {}", fs_desc.used_dirs_count);
        println!();
        
        println!("User Information:");
        println!("----------------");
        println!("Registered Users: {}", fs_desc.users_len);
        println!("Max Users: {}", fs_desc.users.len());
    }
}

impl Cmd for Df {
    fn description(&self) -> String {
        "显示文件系统的磁盘使用情况".to_string()
    }

    fn run(&self, shell: &mut Shell, argv: &[&str]) {
        let mut human_readable = false;
        let mut show_inodes = false;
        let mut show_detailed = false;

        // 解析命令行参数
        for &arg in argv {
            match arg {
                "-h" | "--human-readable" => {
                    human_readable = true;
                }
                "-i" | "--inodes" => {
                    show_inodes = true;
                }
                "-v" | "--verbose" => {
                    show_detailed = true;
                }
                "--help" => {
                    println!("{}", self.help());
                    return;
                }
                arg if arg.starts_with('-') => {
                    println!("df: unknown option '{}'", arg);
                    println!("Try 'df --help' for more information.");
                    return;
                }
                _ => {
                    println!("df: filesystem path not supported");
                    return;
                }
            }
        }

        if show_detailed {
            Self::show_detailed_info(shell);
        } else {
            Self::show_filesystem_info(shell, human_readable, show_inodes);
        }
    }

    fn help(&self) -> String {
        r#"Show filesystem disk space usage

Usage: df [OPTION]...

Options:
  -h, --human-readable  print sizes in human readable format (e.g., 1K 234M 2G)
  -i, --inodes         list inode information instead of block usage
  -v, --verbose        show detailed filesystem information
      --help           display this help and exit

Fields explanation:
  Filesystem    - filesystem name
  Size/1K-blocks - total size (human readable format or 1K blocks)
  Used          - used space
  Avail         - available space
  Use%          - percentage of space used
  Mounted on    - mount point

Examples:
  df              show basic disk usage
  df -h           show in human readable format
  df -i           show inode usage
  df -v           show detailed information"#.to_string()
    }
} 