pub mod project;

use std::path::PathBuf;
use std::{env, fs};

use anyhow::{format_err, Result};

// Find the wrapper in the build directory. This is flaky but for now it will do.
// We need a different binary that will be called by cargo and we can't define a main
// that calls cargo_minver::run_as_compiler_wrapper from an integration test.
pub fn wrapper_path() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let deps_dir = exe_path.parent().unwrap();

    let mut dir_entries = fs::read_dir(deps_dir)?
        .flat_map(|res| res.ok())
        .filter(|e| {
            let file_name = e.file_name();
            let file_name = file_name.to_str().unwrap();
            e.file_type().unwrap().is_file() && file_name.starts_with("minver_wrapper") && !file_name.ends_with(".d")
        })
        .collect::<Vec<_>>();

    dir_entries.sort_unstable_by(|a, b| {
        let modif_time = |e: &fs::DirEntry| e.metadata().unwrap().modified().unwrap();
        modif_time(a).cmp(&modif_time(b))
    });

    dir_entries.iter().last().map(|e| e.path()).ok_or_else(|| format_err!("minver wrapper not found in deps dir"))
}
