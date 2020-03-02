use std::collections::HashMap;
use std::fmt::{self, Display};

use semver::Version;
use serde::{Deserialize, Serialize};

// TODO: allow ignoring features for minimum version calculation?
//       e.g. macro_import_prelude does not seem to be required

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureKind {
    Lang,
    Lib,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub kind: FeatureKind,
    pub since: Option<Version>,
}

impl From<&rustc_feature::Feature> for Feature {
    fn from(feature: &rustc_feature::Feature) -> Self {
        Feature { name: feature.name.to_string(), kind: FeatureKind::Lang, since: Some(feature.since.parse().unwrap()) }
    }
}

impl From<rustc_attr::Stability> for Feature {
    fn from(stab: rustc_attr::Stability) -> Self {
        Feature {
            name: stab.feature.to_string(),
            kind: FeatureKind::Lib,
            since: match stab.level {
                rustc_attr::StabilityLevel::Stable { since } => Some(since.as_str().parse().unwrap()),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub file_name: String,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}:{} {}:{}", self.file_name, self.start_line, self.start_col, self.end_line, self.end_col)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrateAnalysis {
    pub name: String,
    pub features: Vec<Feature>,
    pub uses: HashMap<String, Vec<Span>>,
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
        let mut features = self.crates.iter().map(|a| &a.features).flatten().cloned().collect::<Vec<_>>();

        features.sort_unstable_by(|a, b| if a.since == b.since { a.name.cmp(&b.name) } else { b.since.cmp(&a.since) });
        features.dedup();
        features
    }
}
