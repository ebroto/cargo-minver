use std::path::PathBuf;
use std::{env, fmt, fs};

use anyhow::{format_err, Result};
use tempfile::TempDir;

pub enum Edition {
    Edition2015,
    Edition2018,
}

impl fmt::Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Edition::Edition2015 => write!(f, "2015"),
            Edition::Edition2018 => write!(f, "2018"),
        }
    }
}

pub struct Builder {
    name: String,
    edition: Edition,
    source_files: Vec<PathBuf>,
    abort_on_panic: bool,
}

impl Builder {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), edition: Edition::Edition2015, source_files: Vec::new(), abort_on_panic: false }
    }

    pub fn edition(&mut self, edition: Edition) -> &mut Self {
        self.edition = edition;
        self
    }

    pub fn source_file<P: Into<PathBuf>>(&mut self, path: P) -> Result<&mut Self> {
        let mut target = env::current_dir()?;
        target.push("tests");
        target.push(path.into());

        self.source_files.push(target);
        Ok(self)
    }

    pub fn abort_on_panic(&mut self, value: bool) -> &mut Self {
        self.abort_on_panic = value;
        self
    }

    pub fn create(&mut self) -> Result<Project> {
        if self.source_files.is_empty() {
            format_err!("project must have at least one source file");
        }

        let project_dir = TempDir::new()?;
        let src_dir = project_dir.path().join("src");
        fs::create_dir(&src_dir)?;

        for (num, source_file) in self.source_files.iter().enumerate() {
            let file_name: PathBuf = if num == 0 { "main.rs".into() } else { source_file.file_name().unwrap().into() };

            let mut dest = src_dir.clone();
            dest.push(file_name);
            fs::copy(source_file, dest)?;
        }

        let manifest_path = project_dir.path().join("Cargo.toml");
        let mut manifest = format!(
            r#"[package]
name = "{}"
version = "0.0.0"
edition = "{}"
"#,
            self.name, self.edition
        );

        if self.abort_on_panic {
            manifest.push_str(
                r#"[profile.dev]
panic = "abort"
"#,
            );
        }

        fs::write(manifest_path, manifest)?;

        Ok(Project { _name: self.name.clone(), dir: project_dir })
    }
}

pub struct Project {
    _name: String,
    dir: TempDir,
}

impl Project {
    pub fn _name(&self) -> &str {
        &self._name
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.dir.path().join("Cargo.toml")
    }
}
