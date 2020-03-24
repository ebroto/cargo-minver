use rustc_attr::Stability;
use rustc_feature::ACCEPTED_FEATURES;
use rustc_session::Session;
use rustc_span::{source_map::SourceMap, symbol::Symbol, Span};

use std::collections::{HashMap, HashSet};

use cargo_minver::FeatureKind;

use super::Wrapper;

#[derive(Debug, Default)]
pub struct Context {
    lang_features: HashMap<Symbol, HashSet<Span>>,
    lib_features: HashMap<Stability, HashSet<Span>>,
}

impl Context {
    pub fn record_lang_feature(&mut self, feature: Symbol, span: Span) {
        self.lang_features.entry(feature).or_default().insert(span);
    }

    pub fn record_lib_feature(&mut self, stab: Stability, span: Span) {
        self.lib_features.entry(stab).or_default().insert(span);
    }

    pub fn dump(self, wrapper: &mut Wrapper, session: &Session) {
        let source_map = session.source_map();

        for (feat_name, spans) in self.lang_features {
            let feature = convert_feature(ACCEPTED_FEATURES.iter().find(|f| f.name == feat_name).unwrap());
            wrapper.features.insert(feature);
            wrapper
                .uses
                .entry(feat_name.to_string())
                .or_default()
                .extend(spans.into_iter().map(|s| convert_span(source_map, s)));
        }

        for (stab, spans) in self.lib_features {
            wrapper.features.insert(convert_stability(stab));
            wrapper
                .uses
                .entry(stab.feature.to_string())
                .or_default()
                .extend(spans.into_iter().map(|s| convert_span(source_map, s)));
        }
    }
}

// We can't implement `From` for `Feature` and `Span` because of the orphan rules,
// so the conversions are implemented here as free functions.

fn convert_span(source_map: &SourceMap, span: rustc_span::Span) -> cargo_minver::Span {
    let start = source_map.lookup_char_pos(span.lo());
    let end = source_map.lookup_char_pos(span.hi());

    cargo_minver::Span {
        file_name: start.file.name.to_string(),
        start_line: start.line,
        start_col: start.col.0,
        end_line: end.line,
        end_col: end.col.0,
    }
}

fn convert_feature(feature: &rustc_feature::Feature) -> cargo_minver::Feature {
    cargo_minver::Feature {
        name: feature.name.to_string(),
        kind: FeatureKind::Lang,
        since: Some(feature.since.parse().unwrap()),
    }
}

fn convert_stability(stab: rustc_attr::Stability) -> cargo_minver::Feature {
    cargo_minver::Feature {
        name: stab.feature.to_string(),
        kind: FeatureKind::Lib,
        since: match stab.level {
            rustc_attr::StabilityLevel::Stable { since } => Some(since.as_str().parse().unwrap()),
            _ => None,
        },
    }
}
