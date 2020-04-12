use rustc_attr::Stability;
use rustc_feature::ACCEPTED_FEATURES;
use rustc_session::Session;
use rustc_span::{source_map::SourceMap, symbol::Symbol, Span};

use std::collections::{HashMap, HashSet};

use cargo_minver::{Feature, FeatureKind};

use super::Wrapper;

pub struct StabCtxt<'a> {
    session: &'a Session,
    lang_features: HashMap<Symbol, HashSet<Span>>,
    lib_features: HashMap<Stability, HashSet<Span>>,
}

impl<'a> StabCtxt<'a> {
    pub fn new(session: &'a Session) -> Self {
        Self { session, lang_features: Default::default(), lib_features: Default::default() }
    }

    pub fn record_lang_feature(&mut self, feature: Symbol, span: Span) {
        self.lang_features.entry(feature).or_default().insert(span);
    }

    pub fn record_lib_feature(&mut self, stab: Stability, span: Span) {
        self.lib_features.entry(stab).or_default().insert(span);
    }

    pub fn dump(self, wrapper: &mut Wrapper) {
        macro_rules! add_features {
            ($wrapper: expr, $source_map: expr, $features: expr, $convert_op: expr) => {
                for (elem, spans) in $features {
                    let feature = $convert_op(*elem);
                    $wrapper
                        .uses
                        .entry(feature.name.clone())
                        .or_default()
                        .extend(spans.into_iter().map(|s| convert_span($source_map, *s)));
                    $wrapper.features.insert(feature);
                }
            };
        }

        let source_map = self.session.source_map();
        add_features!(wrapper, source_map, &self.lang_features, convert_lang_feature);
        add_features!(wrapper, source_map, &self.lib_features, convert_lib_feature);
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

fn convert_lang_feature(name: Symbol) -> Feature {
    // Special case the renamed `slice_patterns`.
    fn maybe_min_slice_patterns(name: Symbol) -> Option<Feature> {
        if name.to_string() == "min_slice_patterns" {
            Some(Feature {
                name: "min_slice_patterns".into(),
                kind: FeatureKind::Lang,
                since: Some("1.26.0".parse().unwrap()),
            })
        } else {
            None
        }
    }

    ACCEPTED_FEATURES
        .iter()
        .find(|feat| feat.name == name)
        .map(|feat| Feature {
            name: feat.name.to_string(),
            kind: FeatureKind::Lang,
            since: Some(feat.since.parse().unwrap()),
        })
        .or_else(|| maybe_min_slice_patterns(name))
        .unwrap()
}

fn convert_lib_feature(stab: rustc_attr::Stability) -> Feature {
    Feature {
        name: stab.feature.to_string(),
        kind: FeatureKind::Lib,
        since: match stab.level {
            rustc_attr::StabilityLevel::Stable { since } => Some(since.as_str().parse().unwrap()),
            _ => None,
        },
    }
}
