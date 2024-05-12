use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::canonicalize;
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Output};
use std::process::{ChildStderr, ChildStdout, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};

use nonblock::NonBlockingReader;
use tracing::{error, info, warn};

pub mod api;
pub mod config;

#[derive(Clone)]
pub struct AppState(Arc<Mutex<Backend>>);

impl AppState {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self(Arc::new(Mutex::new(Backend::new(path))))
    }

    pub fn lock(&self) -> MutexGuard<Backend> {
        match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!("Failed to lock AppState: {}", poisoned);
                poisoned.into_inner()
            }
        }
    }
    
    pub fn running(&self) -> bool {
        self.lock().running()
    }

    pub fn path(&self) -> PathBuf {
        self.lock().path()
    }

    pub fn commit_info(&self) -> io::Result<(String, String)> {
        self.lock().commit_info()
    }

    pub fn stdout(&self) -> io::Result<Cow<'static, str>> {
        self.lock().stdout()
    }

    pub fn stderr(&self) -> io::Result<Cow<'static, str>> {
        self.lock().stderr()
    }

    pub fn start(&self) -> io::Result<()> {
        self.lock().start()
    }

    pub fn stop(&self) -> io::Result<Cow<'static, str>> {
        self.lock().stop()
    }

    pub fn restart(&self) -> io::Result<()> {
        self.lock().restart()
    }
}

pub struct Backend {
    process: Option<BackendProcess>,
    path: PathBuf,
}

impl Backend {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            process: None,
            path: path.into(),
        }
    }

    pub fn running(&self) -> bool {
        self.process.is_some()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn commit_info(&self) -> io::Result<(String, String)> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["log", "-n", "1"])
            .output()?;
        Ok((
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }

    pub fn stdout(&mut self) -> io::Result<Cow<'static, str>> {
        match self.process.as_mut() {
            Some(process) => process.stdout().map(Cow::Owned),
            None => {
                warn!("Backend is not running");
                Ok(Cow::Borrowed("Backend is not running"))
            }
        }
    }

    pub fn stderr(&mut self) -> io::Result<Cow<'static, str>> {
        match self.process.as_mut() {
            Some(process) => process.stderr().map(Cow::Owned),
            None => {
                warn!("Backend is not running");
                Ok(Cow::Borrowed("Backend is not running"))
            }
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        if self.running() {
            warn!("Backend is already running");
            return Ok(());
        }
        info!("Installing dependencies");
        match Command::new("mvn")
            .current_dir(&self.path)
            .arg("install")
            .output()
        {
            Ok(output) => {
                let msg = format!(
                    "maven install status: {}\nmaven install stdout: \n{}maven install stderr: \n{}\n",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                info!("{}", msg);
                if !output.status.success() {
                    warn!("Failed to install dependencies");
                    return Err(io::Error::new(io::ErrorKind::Other, msg));
                }
            }
            Err(e) => {
                warn!("Failed to install dependencies: {}", e);
                return Err(e);
            }
        }
        for entry in self.path.join("target").read_dir()?.filter_map(Result::ok) {
            if entry.file_name().to_string_lossy().ends_with(".jar") {
                let jar = canonicalize(entry.path())?;
                info!("Found jar: {}", entry.path().display());
                self.process = Some(BackendProcess::new(jar)?);
                return Ok(());
            }
        }
        warn!("No jar found in target directory");
        Err(io::Error::new(io::ErrorKind::NotFound, "No jar found"))
    }

    pub fn stop(&mut self) -> io::Result<Cow<'static, str>> {
        match self.process.take() {
            Some(process) => {
                let output = process.kill()?;
                let msg = format!(
                    "Backend stopped with status: {}\nstdout: \n{}\nstderr:\n {}\n",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                info!("{}", msg);
                Ok(Cow::Owned(msg))
            }
            None => {
                warn!("Backend is not running");
                Ok(Cow::Borrowed("Backend is not running"))
            }
        }
    }

    pub fn restart(&mut self) -> io::Result<()> {
        self.stop()?;
        self.start()
    }
}

pub struct BackendProcess {
    process: Child,
    stdout: String,
    stdout_rd: NonBlockingReader<ChildStdout>,
    stderr: String,
    stderr_rd: NonBlockingReader<ChildStderr>,
}

impl BackendProcess {
    pub fn new<S: AsRef<OsStr>>(jar: S) -> io::Result<Self> {
        let mut process = Command::new("java")
            .arg("-jar")
            .arg(jar)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Self {
            stdout_rd: NonBlockingReader::from_fd(process.stdout.take().unwrap())?,
            stdout: String::new(),
            stderr_rd: NonBlockingReader::from_fd(process.stderr.take().unwrap())?,
            stderr: String::new(),
            process,
        })
    }

    pub fn kill(self) -> io::Result<Output> {
        Command::new("kill")
            .arg(self.process.id().to_string())
            .output()?;
        self.process.wait_with_output()
    }

    pub fn poll_stdout(&mut self) -> io::Result<Option<String>> {
        let mut output = String::new();
        if self.stdout_rd.read_available_to_string(&mut output)? == 0 {
            Ok(None)
        } else {
            Ok(Some(output))
        }
    }

    pub fn stdout(&mut self) -> io::Result<String> {
        while let Some(output) = self.poll_stdout()? {
            self.stdout.push_str(&output);
        }
        Ok(self.stdout.clone())
    }

    pub fn poll_stderr(&mut self) -> io::Result<Option<String>> {
        let mut output = String::new();
        if self.stderr_rd.read_available_to_string(&mut output)? == 0 {
            Ok(None)
        } else {
            Ok(Some(output))
        }
    }

    pub fn stderr(&mut self) -> io::Result<String> {
        while let Some(output) = self.poll_stderr()? {
            self.stderr.push_str(&output);
        }
        Ok(self.stderr.clone())
    }
}
