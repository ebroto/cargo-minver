use rustc_feature::Feature as RustFeature;

use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureKind {
    Lang,
    Lib,
}

// TODO: allow ignoring features for minimum version calculation
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Feature {
    // TODO: differentiate which crate's `build_script_build` it is
    pub name: String,
    pub kind: FeatureKind,
    pub since: Option<Version>,
}

impl From<&RustFeature> for Feature {
    fn from(feature: &RustFeature) -> Self {
        Feature {
            name: feature.name.to_string(),
            kind: FeatureKind::Lang,
            since: Some(feature.since.parse().unwrap()),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrateAnalysis {
    pub name: String,
    pub features: Vec<Feature>,
}
