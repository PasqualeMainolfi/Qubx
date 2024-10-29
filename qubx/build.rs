use std::process::Command;

#[derive(Debug)]
enum QubxFfmpegError
{
    NotInstalled,
    GenericError,
    InstallationAutoNotAllowed,
    SystemNotSupported
}

fn check_ffmpeg() -> Result<(), QubxFfmpegError> {
    println!("cargo:warning=Checking ffmpeg installation...");
    let com = Command::new("ffmpeg").arg("-version").output();
    match com {
        Ok(out) => {
            if out.status.success() { 
                println!("cargo:warning=ffmpeg status [installed]");
                Ok(()) 
            } else {
                println!("cargo:warning=ffmpeg check failed with error status"); 
                Err(QubxFfmpegError::GenericError) 
            }
        }
        Err(_) => {
            println!("cargo:warning=ffmpeg not status [not installed]");
            Err(QubxFfmpegError::NotInstalled)}
    }
}

fn install_ffmpeg() -> Result<(), QubxFfmpegError> {
    println!("cargo:warning=Installing ffmpeg...");
    if cfg!(target_os = "linux") {
        Command::new("sudo").arg("apt").arg("update").status().unwrap();
        Command::new("sudo").arg("apt").arg("install").arg("-y").arg("ffmpeg").status().unwrap();
    } else if cfg!(target_os = "macos") {
        let brew = Command::new("brew").arg("--version").output().unwrap();
        if !brew.status.success() { 
            println!("cargo:warning=It is not possible to install ffmpeg automatically without Homebrew. Please install brew from <https://brew.sh/>");
            return Err(QubxFfmpegError::InstallationAutoNotAllowed)
        }
        Command::new("brew").arg("install").arg("ffmpeg").status().unwrap();
    } else if cfg!(target_os = "windows") {
        println!("cargo:warning=It is not possible to install ffmpeg automatically on windows. Please install ffmpeg from <https://ffmpeg.org/download.html> and add to PATH");
        return Err(QubxFfmpegError::InstallationAutoNotAllowed)
    } else {
        println!("cargo:warning=System not supported");
        return Err(QubxFfmpegError::SystemNotSupported)
    }
    Ok(())
}

fn ensure_ffmpeg() -> Result<(), QubxFfmpegError>{
    let ffmpeg_installed = check_ffmpeg();
    match ffmpeg_installed {
        Ok(()) => Ok(()),
        Err(_) => {
            println!("cargo:warning=Trying to install ffmpeg...");
            match install_ffmpeg() {
                Ok(()) => Ok(()),
                Err(_) => {
                    println!("cargo:warning=ffmpeg installation failed...");
                    Err(QubxFfmpegError::InstallationAutoNotAllowed)
                }
            }
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:warning=Check system...");
    ensure_ffmpeg().unwrap();
    println!("cargo:warning=Done!");
}