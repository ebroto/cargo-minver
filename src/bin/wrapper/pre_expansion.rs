use rustc_ast::{ast, visit};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_session::Session;
use rustc_span::symbol::{sym, Symbol};
use rustc_span::Span;

use std::collections::{HashMap, HashSet};

use super::{convert_feature, convert_span, Wrapper};

#[derive(Debug, Default)]
struct Visitor {
    lang_features: HashMap<Symbol, HashSet<Span>>,
}

impl visit::Visitor<'_> for Visitor {
    fn visit_attribute(&mut self, attr: &ast::Attribute) {
        if let ast::AttrKind::Normal(_) = &attr.kind {
            if attr.has_name(sym::cfg) {
                for item in attr.meta_item_list().unwrap_or_default() {
                    // NOTE: sym::target_vendor does not exist
                    let name = item.name_or_empty();
                    if name == sym::doctest {
                        self.record_lang_feature(sym::cfg_doctest, attr.span);
                    } else if name.as_str() == "target_vendor" {
                        self.record_lang_feature(sym::cfg_target_vendor, attr.span);
                    }
                }
            }

            if let Some(meta) = attr.meta() {
                self.check_cfg_attr_multi(&meta);
            }
        }

        visit::walk_attribute(self, attr);
    }

    fn visit_mac(&mut self, _mac: &ast::Mac) {
        // Do nothing.
    }
}

impl Visitor {
    fn record_lang_feature(&mut self, feature: Symbol, span: Span) {
        self.lang_features.entry(feature).or_default().insert(span);
    }

    fn check_cfg_attr_multi(&mut self, meta: &ast::MetaItem) {
        if meta.name_or_empty() != sym::cfg_attr {
            return;
        }

        if let Some(metas) = meta.meta_item_list() {
            if metas.len() != 2 {
                self.record_lang_feature(sym::cfg_attr_multi, meta.span);
            }

            for meta in metas {
                if let Some(meta) = meta.meta_item() {
                    self.check_cfg_attr_multi(&meta);
                }
            }
        }
    }
}

pub fn walk_crate(wrapper: &mut Wrapper, krate: &ast::Crate, session: &Session) {
    let mut visitor = Visitor::default();
    visit::walk_crate(&mut visitor, &krate);

    let source_map = session.source_map();
    for (feat_name, spans) in visitor.lang_features {
        let feature = convert_feature(ACCEPTED_FEATURES.iter().find(|f| f.name == feat_name).unwrap());
        wrapper.features.insert(feature);
        wrapper
            .uses
            .entry(feat_name.to_string())
            .or_default()
            .extend(spans.into_iter().map(|s| convert_span(source_map, s)));
    }
}
