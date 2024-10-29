use std::process::Command;
use std::sync::Once;

static INIT_FFMPEG: Once = Once::new();

#[derive(Debug)]
pub enum QubxFfmpegError
{
    NotInstalled,
    GenericError,
    InstallationAutoNotAllowed,
    SystemNotSupported
}

fn check_ffmpeg() -> Result<(), QubxFfmpegError> {
    let com = Command::new("ffmpeg").arg("-version").output();
    match com {
        Ok(out) => if out.status.success() { Ok(()) } else { Err(QubxFfmpegError::GenericError) }
        Err(_) => {
            println!("[WARNING] ffmpeg not installed!");
            Err(QubxFfmpegError::NotInstalled)}
    }
}

fn install_ffmpeg() -> Result<(), QubxFfmpegError> {
    if cfg!(target_os = "linux") {
        println!("[INFO] Installing ffmpeg...");
        Command::new("sudo").arg("apt").arg("update").status().unwrap();
        Command::new("sudo").arg("apt").arg("install").arg("-y").arg("ffmpeg").status().unwrap();
    } else if cfg!(target_os = "macos") {
        let brew = Command::new("brew").arg("--version").output().unwrap();
        if !brew.status.success() { 
            println!("[ERROR] It is not possible to install ffmpeg automatically without Homebrew. Please install brew from <https://brew.sh/>");
            return Err(QubxFfmpegError::InstallationAutoNotAllowed)
        }
        println!("[INFO] Installing ffmpeg...");
        Command::new("brew").arg("install").arg("ffmpeg").status().unwrap();
    } else if cfg!(target_os = "windows") {
        println!("[ERROR] It is not possible to install ffmpeg automatically on windows. Please install ffmpeg from <https://ffmpeg.org/download.html> and add to PATH");
        return Err(QubxFfmpegError::InstallationAutoNotAllowed)
    } else {
        return Err(QubxFfmpegError::SystemNotSupported)
    }
    Ok(())
}

pub(crate) fn ensure_ffmpeg() {
    INIT_FFMPEG.call_once(|| {
        println!("[INFO] Check ffmpeg status...");
        let ffmpeg_installed = check_ffmpeg();
        match ffmpeg_installed {
            Ok(()) => {
                println!("[INFO] ffmpeg status: OK")
            },
            Err(_) => {
                println!("[INFO] Trying to install ffmpeg...");
                install_ffmpeg().unwrap();
                println!("[INFO] ffmpeg status: OK");
                println!("[INFO] Rebuild and run the code...");
            }
        }
    })
}