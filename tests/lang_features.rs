mod util;

use anyhow::Result;

use cargo_minver::{Driver, FeatureKind};
use util::project::Builder;

// TODO: use a lang_feature_test!() macro to avoid repetition
// NOTE: the server port must be different in every test because they run in parallel

#[test]
fn dotdot_in_tuple_patterns() -> Result<()> {
    let project = Builder::new("dotdot_in_tuple_patterns")
        .with_source_file("lang_features/dotdot_in_tuple_patterns.rs")?
        .create()?;

    let analysis = Driver::new()
        .server_port(42001)
        .wrapper_path(util::wrapper_path()?)
        .manifest_path(project.manifest_path())
        .quiet(true)
        .execute()?;

    let feature = analysis.all_features().into_iter().find(|f| f.name == "dotdot_in_tuple_patterns").unwrap();
    assert_eq!(FeatureKind::Lang, feature.kind);
    assert_eq!(Some("1.14.0".parse().unwrap()), feature.since);

    let mut uses = analysis.all_feature_uses("dotdot_in_tuple_patterns");
    uses.sort();
    assert_eq!(6, uses.len());
    assert_eq!("src/main.rs 4:8 4:20", format!("{}", uses[0]));
    assert_eq!("src/main.rs 5:8 5:20", format!("{}", uses[1]));
    assert_eq!("src/main.rs 6:8 6:20", format!("{}", uses[2]));
    assert_eq!("src/main.rs 13:8 13:24", format!("{}", uses[3]));
    assert_eq!("src/main.rs 14:8 14:24", format!("{}", uses[4]));
    assert_eq!("src/main.rs 15:8 15:24", format!("{}", uses[5]));

    Ok(())
}

#[test]
fn loop_break_value() -> Result<()> {
    let project = Builder::new("loop_break_value").with_source_file("lang_features/loop_break_value.rs")?.create()?;

    let analysis = Driver::new()
        .server_port(42005)
        .wrapper_path(util::wrapper_path()?)
        .manifest_path(project.manifest_path())
        .quiet(true)
        .execute()?;

    let feature = analysis.all_features().into_iter().find(|f| f.name == "loop_break_value").unwrap();
    assert_eq!(FeatureKind::Lang, feature.kind);
    assert_eq!(Some("1.19.0".parse().unwrap()), feature.since);

    let mut uses = analysis.all_feature_uses("loop_break_value");
    uses.sort();
    assert_eq!(1, uses.len());
    assert_eq!("src/main.rs 3:8 3:16", format!("{}", uses[0]));

    Ok(())
}

#[test]
fn dotdoteq_in_patterns() -> Result<()> {
    let project =
        Builder::new("dotdoteq_in_patterns").with_source_file("lang_features/dotdoteq_in_patterns.rs")?.create()?;

    let analysis = Driver::new()
        .server_port(42002)
        .wrapper_path(util::wrapper_path()?)
        .manifest_path(project.manifest_path())
        .quiet(true)
        .execute()?;

    let feature = analysis.all_features().into_iter().find(|f| f.name == "dotdoteq_in_patterns").unwrap();
    assert_eq!(FeatureKind::Lang, feature.kind);
    assert_eq!(Some("1.26.0".parse().unwrap()), feature.since);

    let uses = analysis.all_feature_uses("dotdoteq_in_patterns");
    assert_eq!(1, uses.len());
    assert_eq!("src/main.rs 3:8 3:14", format!("{}", uses[0]));

    Ok(())
}

#[test]
fn inclusive_range_syntax() -> Result<()> {
    let project =
        Builder::new("inclusive_range_syntax").with_source_file("lang_features/inclusive_range_syntax.rs")?.create()?;

    let analysis = Driver::new()
        .server_port(42003)
        .wrapper_path(util::wrapper_path()?)
        .manifest_path(project.manifest_path())
        .quiet(true)
        .execute()?;

    let feature = analysis.all_features().into_iter().find(|f| f.name == "inclusive_range_syntax").unwrap();
    assert_eq!(FeatureKind::Lang, feature.kind);
    assert_eq!(Some("1.26.0".parse().unwrap()), feature.since);

    let uses = analysis.all_feature_uses("inclusive_range_syntax");
    assert_eq!(1, uses.len());
    assert_eq!("src/main.rs 2:18 2:23", format!("{}", uses[0]));

    Ok(())
}

#[test]
fn crate_in_paths() -> Result<()> {
    let project = Builder::new("crate_in_paths").with_source_file("lang_features/crate_in_paths.rs")?.create()?;

    let analysis = Driver::new()
        .server_port(42004)
        .wrapper_path(util::wrapper_path()?)
        .manifest_path(project.manifest_path())
        .quiet(true)
        .execute()?;

    let feature = analysis.all_features().into_iter().find(|f| f.name == "crate_in_paths").unwrap();
    assert_eq!(FeatureKind::Lang, feature.kind);
    assert_eq!(Some("1.30.0".parse().unwrap()), feature.since);

    let mut uses = analysis.all_feature_uses("crate_in_paths");
    uses.sort();
    assert_eq!(9, uses.len());
    assert_eq!("src/main.rs 12:4 12:9", format!("{}", uses[0]));
    assert_eq!("src/main.rs 14:12 14:17", format!("{}", uses[1]));
    assert_eq!("src/main.rs 16:8 16:13", format!("{}", uses[2]));
    assert_eq!("src/main.rs 20:12 20:17", format!("{}", uses[3]));
    assert_eq!("src/main.rs 21:11 21:16", format!("{}", uses[4]));
    assert_eq!("src/main.rs 25:8 25:13", format!("{}", uses[5]));
    assert_eq!("src/main.rs 27:9 27:14", format!("{}", uses[6]));
    assert_eq!("src/main.rs 35:26 35:31", format!("{}", uses[7]));
    assert_eq!("src/main.rs 35:45 35:50", format!("{}", uses[8]));

    Ok(())
}
