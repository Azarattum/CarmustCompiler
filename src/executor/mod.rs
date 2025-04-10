use crate::compile::CompileError;
use std::{
    env::temp_dir,
    fs::{metadata, remove_file, set_permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
};

pub trait Executable {
    fn execute(&self, filename: &str) -> Result<i32, CompileError>;
}

impl Executable for String {
    fn execute(&self, filename: &str) -> Result<i32, CompileError> {
        let object_file = temp_dir().join("program.tmp.o");
        let executable_file = filename.replace(".c", "");

        let mut clang_process = Command::new("clang")
            .arg("-x")
            .arg("assembler")
            .arg("-o")
            .arg(object_file.clone())
            .arg("-c")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()
            .or_else(|_| {
                Err(CompileError {
                    message: "Failed to start `clang` process!",
                })
            })?;

        clang_process
            .stdin
            .as_mut()
            .ok_or(CompileError {
                message: "Failed to open stdin!",
            })?
            .write(self.as_bytes())
            .or_else(|_| {
                Err(CompileError {
                    message: "Failed to write to stdin!",
                })
            })?;

        let status = clang_process.wait().or_else(|_| {
            Err(CompileError {
                message: "Failed to wait on `clang` process!",
            })
        })?;

        if !status.success() {
            return Err(CompileError {
                message: "`clang` command failed!",
            });
        }

        let status = Command::new("ld")
            .arg(object_file.clone())
            .arg("-e")
            .arg("main")
            .arg("-arch")
            .arg("arm64")
            .arg("-o")
            .arg(executable_file.clone())
            .status()
            .or_else(|_| {
                Err(CompileError {
                    message: "Failed to execute `ld` command!",
                })
            })?;

        if !status.success() {
            return Err(CompileError {
                message: "`ld` command failed!",
            });
        }

        let mut permissions = metadata(executable_file.clone())
            .or_else(|_| {
                Err(CompileError {
                    message: "Failed to get metadata for output file!",
                })
            })?
            .permissions();
        permissions.set_mode(0o755);
        set_permissions(executable_file.clone(), permissions).or_else(|_| {
            Err(CompileError {
                message: "Failed to set permissions for output file!",
            })
        })?;

        let exec_status = Command::new(executable_file.clone())
            .status()
            .or_else(|_| {
                Err(CompileError {
                    message: "Failed to execute the output file!",
                })
            })?;

        remove_file(object_file).or_else(|_| {
            Err(CompileError {
                message: "Failed to remove temporary object file!",
            })
        })?;

        Ok(exec_status.code().unwrap_or(0))
    }
}
