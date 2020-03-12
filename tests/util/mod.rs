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

macro_rules! test_lang_feature {
    ($port: expr, ($name: ident, $edition: expr, $version: expr, $spans: expr $(, $inspect:expr)?)) => {
        #[test]
        fn $name() -> anyhow::Result<()> {
            let name = stringify!($name);
            let source_file = format!("lang_files/{}.rs", name);
            let project = util::project::Builder::new(name) //
                .with_edition($edition)
                .with_source_file(source_file)?
                .create()?;

            let analysis = cargo_minver::Driver::new()
                .server_port($port)
                .wrapper_path(util::wrapper_path()?)
                .manifest_path(project.manifest_path())
                .quiet(true)
                .execute()?;

            let feature = analysis.feature(name).expect("feature not found");
            assert_eq!(cargo_minver::FeatureKind::Lang, feature.kind, "expected feature kind to match");
            assert_eq!(Some($version.parse().unwrap()), feature.since, "expected stabilization version to match");

            let uses = analysis.all_feature_uses(name);
            $(if ($inspect) { dbg!(&uses); })?
            assert_eq!($spans.len(), uses.len(), "expected feature use count to match");
            for (expected, actual) in $spans.iter().zip(uses.iter()) {
                assert_eq!(format!("src/main.rs {}", expected), format!("{}", actual), "expected span to match");
            }
            Ok(())
        }
    };
}

macro_rules! test_lang_features {
    ($($feature:tt),*) => {
        test_lang_features!(@step 42000u16, $($feature,)*);
    };

    (@step $port:expr, $head:tt, $($tail:tt,)*) => {
        test_lang_feature!($port, $head);
        test_lang_features!(@step $port + 1u16, $($tail,)*);
    };

    (@step $_port: expr,) => {};
}
