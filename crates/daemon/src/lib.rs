use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::canonicalize;
use std::io::{self, Read, Stderr, Stdout};
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
        if self.process.is_none() {
            warn!("Backend is not running");
            return Ok(Cow::Borrowed("Backend is not running"));
        }
        let stdout = self.process.as_mut().unwrap().stdout.take();
        match stdout {
            Some(mut stdout) => {
                let mut output = String::new();
                info!(
                    "Read {} bytes from backend stdout",
                    stdout.read_to_string(&mut output)?
                );
                Ok(Cow::Owned(output))
            }
            None => {
                warn!("Backend is running but has no stdout");
                Ok(Cow::Borrowed("Backend is running but has no stdout"))
            }
        }
    }

    pub fn stderr(&mut self) -> io::Result<Cow<'static, str>> {
        if self.process.is_none() {
            warn!("Backend is not running");
            return Ok(Cow::Borrowed("Backend is not running"));
        }
        let stderr = self.process.as_mut().unwrap().stderr.take();
        match stderr {
            Some(mut stderr) => {
                let mut output = String::new();
                info!(
                    "Read {} bytes from backend stderr",
                    stderr.read_to_string(&mut output)?
                );
                Ok(Cow::Owned(output))
            }
            None => {
                warn!("Backend is running but has no stderr");
                Ok(Cow::Borrowed("Backend is running but has no stderr"))
            }
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        if self.process.is_some() {
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
    stdout: NonBlockingReader<ChildStdout>,
    stderr: NonBlockingReader<ChildStderr>,
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
            stdout: NonBlockingReader::from_fd(process.stdout.take().unwrap())?,
            stderr: NonBlockingReader::from_fd(process.stderr.take().unwrap())?,
            process,
        })
    }

    pub fn kill(mut self) -> io::Result<Output> {
        self.process.kill()?;
        self.process.wait_with_output()
    }
}
