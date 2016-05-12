use std::process::{Command,ExitStatus};
use std::io;
use std::result;

#[derive(Debug)]
pub struct GitWrapper {
    repo: String,
    sign: bool,
}

pub type Result<T> = result::Result<T, io::Error>;

/// Version control trait. Note that `add` and `remove` will not commit the 
/// operation. Hence `commit` has to be called separatly.
pub trait VersionControl {
    fn add(&self, file: &str) -> Result<ExitStatus>; 
    fn remove(&self, file: &str) -> Result<ExitStatus>;
    fn commit(&self, message: &str) -> Result<ExitStatus>;
    fn cmd_dispatch(&self, args: Vec<&str>) -> Result<ExitStatus>;
}

impl GitWrapper {
    pub fn new(repo_path: &str) -> GitWrapper {
        let repo_path = String::from(repo_path);
        let output = Command::new("git")
                        .arg("config")
                        .arg("--bool")
                        .arg("--get")
                        .arg("pass.signcommits")
                        .current_dir(&repo_path)
                        .output();

        let sign = match output {
            Ok(output) => {
                match String::from_utf8_lossy(&output.stdout).trim() {
                    "true" | "True" | "TRUE" => true,
                    _ => false
                }
            },
            Err(_) => false
        };

        GitWrapper {
            repo: repo_path,
            sign: sign,
        }
    }
}


impl VersionControl for GitWrapper {
    fn add(&self, file: &str) -> Result<ExitStatus> {
        Command::new("git")
            .arg("add")
            .arg(file)
            .current_dir(&self.repo)
            .status()
    }

    fn commit(&self, message: &str) -> Result<ExitStatus> {
        let mut cmd = Command::new("git");
        cmd.arg("commit")
           .arg("-m")
           .arg(message)
           .current_dir(&self.repo);
        if self.sign {
            cmd.arg("-S");
        }
        cmd.status()
    }

    fn remove(&self, file: &str) -> Result<ExitStatus> {
        let mut cmd = Command::new("git");
        cmd.arg("rm")
           .arg("-qr")
           .arg(file)
           .current_dir(&self.repo);
        cmd.status()
    }

    fn cmd_dispatch(&self, args: Vec<&str>) -> Result<ExitStatus> {
        let mut cmd = Command::new("git");
        cmd.args(args.as_slice())
           .current_dir(&self.repo);
        cmd.status()
    }
}
