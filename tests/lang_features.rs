mod util;

use anyhow::Result;

use cargo_minver::{Driver, FeatureKind};
use util::project::Builder;

#[test]
fn dotdot_in_tuple_patterns() -> Result<()> {
    let project = Builder::new("dotdot_in_tuple_patterns")
        .with_source_file("lang_features/dotdot_in_tuple_patterns.rs")?
        .create()?;

    let analysis = Driver::new() //
        .wrapper_path(util::wrapper_path()?)
        .manifest_path(project.manifest_path())
        .execute()?;

    let feature = analysis.all_features().into_iter().find(|f| f.name == "dotdot_in_tuple_patterns").unwrap();
    assert_eq!(FeatureKind::Lang, feature.kind);
    assert_eq!(Some("1.14.0".parse().unwrap()), feature.since);

    let mut uses = analysis.all_feature_uses("dotdot_in_tuple_patterns");
    uses.sort_unstable_by(|a, b| a.start_line.cmp(&b.start_line));
    assert_eq!("src/main.rs 4:8 4:20", format!("{}", uses[0]));
    assert_eq!("src/main.rs 5:8 5:20", format!("{}", uses[1]));
    assert_eq!("src/main.rs 6:8 6:20", format!("{}", uses[2]));
    assert_eq!("src/main.rs 13:8 13:24", format!("{}", uses[3]));
    assert_eq!("src/main.rs 14:8 14:24", format!("{}", uses[4]));
    assert_eq!("src/main.rs 15:8 15:24", format!("{}", uses[5]));

    Ok(())
}
