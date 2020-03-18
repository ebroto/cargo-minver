use rustc_ast::{ast, visit};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_parse::{self, MACRO_ARGUMENTS};
use rustc_session::{parse::ParseSess, Session};
use rustc_span::symbol::{sym, Symbol};
use rustc_span::Span;

use std::collections::{HashMap, HashSet};

use super::{convert_feature, convert_span, Wrapper};

// NOTE: Active attributes are removed after expansion so we need to catch them here.
struct Visitor<'a> {
    lang_features: HashMap<Symbol, HashSet<Span>>,
    parse_sess: &'a ParseSess,
    // NOTE: sym::target_vendor does not exist
    target_vendor: Symbol,
}

impl<'a, 'b> visit::Visitor<'b> for Visitor<'a> {
    fn visit_attribute(&mut self, attr: &ast::Attribute) {
        if attr.has_name(sym::cfg) || attr.has_name(sym::cfg_attr) {
            if let Some(ref item) = attr.meta() {
                self.walk_cfg_metas(item);
            }
        }

        visit::walk_attribute(self, attr);
    }

    fn visit_mac(&mut self, mac: &ast::Mac) {
        if mac.path.segments.len() == 1 && mac.path.segments[0].ident.name == sym::cfg {
            let tts = mac.args.inner_tokens();
            let mut parser = rustc_parse::stream_to_parser(self.parse_sess, tts, MACRO_ARGUMENTS);
            if let Ok(cfg) = parser.parse_meta_item() {
                self.walk_cfg_metas(&cfg);
            }
        }

        visit::walk_mac(self, mac);
    }

    fn visit_param(&mut self, param: &ast::Param) {
        if !param.attrs.is_empty() {
            self.record_lang_feature(sym::param_attrs, param.span);
        }

        visit::walk_param(self, param);
    }

    fn visit_generic_param(&mut self, param: &ast::GenericParam) {
        if !param.attrs.is_empty() {
            self.record_lang_feature(sym::generic_param_attrs, param.attrs[0].span);
        }
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        if let ast::ExprKind::Struct(_, fields, _) = &expr.kind {
            if fields.iter().any(|f| !f.attrs.is_empty()) {
                self.record_lang_feature(sym::struct_field_attributes, expr.span)
            }
        }

        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &ast::Pat) {
        if let ast::PatKind::Struct(_, fields, _) = &pat.kind {
            if fields.iter().any(|f| !f.attrs.is_empty()) {
                self.record_lang_feature(sym::struct_field_attributes, pat.span)
            }
        }

        visit::walk_pat(self, pat);
    }
}

impl<'a> Visitor<'a> {
    fn new(parse_sess: &'a ParseSess) -> Self {
        Self { lang_features: Default::default(), parse_sess, target_vendor: Symbol::intern("target_vendor") }
    }

    fn record_lang_feature(&mut self, feature: Symbol, span: Span) {
        self.lang_features.entry(feature).or_default().insert(span);
    }

    fn walk_cfg_metas(&mut self, item: &ast::MetaItem) {
        match &item.kind {
            ast::MetaItemKind::List(items) => {
                let name = item.name_or_empty();
                if name == sym::cfg_attr && items.len() != 2 {
                    self.record_lang_feature(sym::cfg_attr_multi, item.span);
                }

                for nested in items {
                    if let Some(item) = nested.meta_item() {
                        self.walk_cfg_metas(item);
                    }
                }
            },
            _ => {
                self.visit_cfg_meta(&item);
            },
        }
    }

    fn visit_cfg_meta(&mut self, item: &ast::MetaItem) {
        let maybe_feature = match item.name_or_empty() {
            sym::doctest => Some(sym::cfg_doctest),
            sym::target_feature => Some(sym::cfg_target_feature),
            vendor if vendor == self.target_vendor => Some(sym::cfg_target_vendor),
            _ => None,
        };

        if let Some(feature) = maybe_feature {
            self.record_lang_feature(feature, item.span);
        }
    }
}

pub fn walk_crate(wrapper: &mut Wrapper, krate: &ast::Crate, session: &Session) {
    let mut visitor = Visitor::new(&session.parse_sess);
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
