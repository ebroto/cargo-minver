use rustc_feature::Feature as RustFeature;

use std::convert::TryFrom;

use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    // TODO: differentiate which crate's `build_script_build` it is
    pub name: String,
    pub since: Version,
}

impl TryFrom<&RustFeature> for Feature {
    type Error = anyhow::Error;

    fn try_from(feature: &RustFeature) -> Result<Self> {
        let name = feature.name.to_string();
        let since = feature.since.parse()?;
        Ok(Feature { name, since })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrateAnalysis {
    pub name: String,
    pub features: Vec<Feature>,
}
