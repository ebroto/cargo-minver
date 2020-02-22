use rustc_attr::{Stability, StabilityLevel};
use rustc_feature::Feature as LangFeature;

use semver::Version;
use serde::{Deserialize, Serialize};

// TODO: allow ignoring features for minimum version calculation?
//       e.g. macro_import_prelude does not seem to be required

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureKind {
    Lang,
    Lib,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Feature {
    pub name: String,
    pub kind: FeatureKind,
    pub since: Option<Version>,
}

impl From<&LangFeature> for Feature {
    fn from(feature: &LangFeature) -> Self {
        Feature {
            name: feature.name.to_string(),
            kind: FeatureKind::Lang,
            since: Some(feature.since.parse().unwrap()),
        }
    }
}

impl From<Stability> for Feature {
    fn from(stab: Stability) -> Self {
        Feature {
            name: stab.feature.to_string(),
            kind: FeatureKind::Lib,
            since: if let StabilityLevel::Stable { since } = stab.level {
                Some(since.as_str().parse().unwrap())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrateAnalysis {
    pub name: String,
    pub features: Vec<Feature>,
}

#[derive(Debug)]
pub struct Analysis {
    crates: Vec<CrateAnalysis>,
}

impl From<Vec<CrateAnalysis>> for Analysis {
    fn from(crates: Vec<CrateAnalysis>) -> Self {
        Self { crates }
    }
}

impl Analysis {
    pub fn all_features(&self) -> Vec<Feature> {
        let mut features = self
            .crates
            .iter()
            .map(|a| &a.features)
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        features.sort_unstable_by(|a, b| {
            if a.since == b.since {
                a.name.cmp(&b.name)
            } else {
                b.since.cmp(&a.since)
            }
        });
        features.dedup();
        features
    }
}
