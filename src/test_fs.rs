use std::path::PathBuf;
use anyhow::Result;
use log::info;
use std::fs::{self, File};
use std::io::Write;
use std::time::Duration;
use crate::fs::FsManager;
use tokio::process::Command;

pub async fn run_fs_tests() -> Result<()> {
    info!("开始文件系统测试...");
    
    // 创建测试目录
    let test_dir = PathBuf::from("test_files");
    fs::create_dir_all(&test_dir)?;
    
    // 创建测试文件
    create_test_files(&test_dir)?;
    
    // 创建文件系统管理器
    let fs_manager = FsManager::new(5, Duration::from_millis(100));
    
    // 测试删除只读文件
    info!("测试删除只读文件...");
    fs_manager.force_remove_file(test_dir.join("readonly.txt")).await?;
    
    // 测试删除被占用的文件
    info!("测试删除被占用的文件...");
    fs_manager.force_remove_file(test_dir.join("locked.txt")).await?;
    
    // 测试删除隐藏文件
    info!("测试删除隐藏文件...");
    fs_manager.force_remove_file(test_dir.join(".hidden.txt")).await?;

    // 测试删除运行中的Python脚本
    info!("测试删除运行中的Python脚本...");
    test_running_python(&test_dir, &fs_manager).await?;

    // 测试删除Git仓库中的文件
    info!("测试删除Git仓库中的文件...");
    test_git_repo(&test_dir, &fs_manager).await?;
    
    // 测试删除特殊权限目录
    info!("测试删除特殊权限目录...");
    fs_manager.force_remove_dir_all(&test_dir).await?;
    
    info!("文件系统测试完成");
    Ok(())
}

async fn test_running_python(test_dir: &PathBuf, fs_manager: &FsManager) -> Result<()> {
    // 创建一个简单的Python脚本
    let script_path = test_dir.join("test_script.py");
    let mut file = File::create(&script_path)?;
    file.write_all(b"
import time
while True:
    print('Running...')
    time.sleep(1)
")?;

    // 启动Python脚本
    info!("启动Python脚本...");
    let mut child = Command::new("python")
        .arg(&script_path)
        .spawn()?;

    // 等待脚本运行一会
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 尝试删除正在运行的脚本
    info!("尝试删除运行中的脚本...");
    fs_manager.force_remove_file(&script_path).await?;

    // 确保进程已经终止
    if let Ok(Some(status)) = child.try_wait() {
        info!("Python进程已终止，状态: {:?}", status);
    } else {
        child.kill().await?;
        info!("手动终止Python进程");
    }

    Ok(())
}

async fn test_git_repo(test_dir: &PathBuf, fs_manager: &FsManager) -> Result<()> {
    // 创建Git仓库
    let repo_dir = test_dir.join("git_test");
    fs::create_dir_all(&repo_dir)?;

    // 初始化Git仓库
    info!("初始化Git仓库...");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_dir)
        .output()?;

    // 创建测试文件
    let test_file = repo_dir.join("test.txt");
    let mut file = File::create(&test_file)?;
    file.write_all(b"Test content")?;

    // 添加文件到Git
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&repo_dir)
        .output()?;

    // 创建提交
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_dir)
        .output()?;

    // 模拟Git操作
    let mut child = Command::new("git")
        .args(["status"])
        .current_dir(&repo_dir)
        .spawn()?;

    // 尝试删除文件
    info!("尝试删除Git仓库中的文件...");
    fs_manager.force_remove_file(&test_file).await?;

    // 确保Git进程已经终止
    child.kill().await?;

    // 删除整个仓库
    info!("删除Git仓库...");
    fs_manager.force_remove_dir_all(&repo_dir).await?;

    Ok(())
}

fn create_test_files(test_dir: &PathBuf) -> Result<()> {
    // 创建只读文件
    let readonly_path = test_dir.join("readonly.txt");
    let mut file = File::create(&readonly_path)?;
    file.write_all(b"This is a readonly file")?;
    let mut perms = fs::metadata(&readonly_path)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&readonly_path, perms)?;
    
    // 创建被占用的文件
    let locked_path = test_dir.join("locked.txt");
    let mut file = File::create(&locked_path)?;
    file.write_all(b"This is a locked file")?;
    
    // 创建隐藏文件
    let hidden_path = test_dir.join(".hidden.txt");
    let mut file = File::create(&hidden_path)?;
    file.write_all(b"This is a hidden file")?;
    
    #[cfg(windows)]
    {
        use std::process::Command;
        // 设置文件为隐藏
        Command::new("attrib")
            .args(["+H", hidden_path.to_str().unwrap()])
            .output()?;
    }
    
    // 创建特殊权限目录
    let special_dir = test_dir.join("special_dir");
    fs::create_dir(&special_dir)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&special_dir)?.permissions();
        perms.set_mode(0o000); // 移除所有权限
        fs::set_permissions(&special_dir, perms)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fs_operations() -> Result<()> {
        run_fs_tests().await
    }
} 