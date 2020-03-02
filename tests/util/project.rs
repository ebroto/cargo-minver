use std::path::PathBuf;
use std::{env, fs};

use anyhow::{format_err, Result};
use tempfile::TempDir;

pub struct Builder {
    name: String,
    source_files: Vec<PathBuf>,
}

impl Builder {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), source_files: Vec::new() }
    }

    pub fn with_source_file<P: Into<PathBuf>>(&mut self, path: P) -> Result<&mut Self> {
        let mut target = env::current_dir()?;
        target.push("tests");
        target.push(path.into());

        self.source_files.push(target);
        Ok(self)
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
        let manifest = format!(
            r#"[package]
name = "{}"
version = "0.0.0"
"#,
            self.name
        );
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
