use rustc_ast::ast::{self, Pat, PatKind, RangeEnd, RangeSyntax};
use rustc_ast::ptr::P;
use rustc_ast::visit::{self, FnKind};
use rustc_attr::{Stability, StabilityLevel};
use rustc_feature::ACCEPTED_FEATURES;
use rustc_resolve::{ParentScope, Resolver};
use rustc_session::Session;
use rustc_span::source_map::{SourceMap, Spanned};
use rustc_span::symbol::{kw, sym, Symbol};
use rustc_span::Span;

use std::collections::{HashMap, HashSet};

use super::{convert_feature, convert_span, Wrapper};

#[derive(Debug, Default)]
struct Visitor {
    lang_features: HashMap<Symbol, HashSet<Span>>,
}

impl visit::Visitor<'_> for Visitor {
    fn visit_fn(&mut self, fn_kind: FnKind, span: Span, _node_id: ast::NodeId) {
        if let Some(header) = fn_kind.header() {
            if header.asyncness.is_async() {
                self.record_lang_feature(sym::async_await, span);
            }
        }

        visit::walk_fn(self, fn_kind, span);
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        match &expr.kind {
            ast::ExprKind::Range(_, _, ast::RangeLimits::Closed) => {
                self.record_lang_feature(sym::inclusive_range_syntax, expr.span);
            },
            ast::ExprKind::Break(_, Some(_)) => {
                self.record_lang_feature(sym::loop_break_value, expr.span);
            },
            ast::ExprKind::Async(..) => {
                self.record_lang_feature(sym::async_await, expr.span);
            },
            ast::ExprKind::Await(_) => {
                self.record_lang_feature(sym::async_await, expr.span);
            },
            _ => {},
        }

        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &ast::Pat) {
        fn has_rest(ps: &[P<Pat>]) -> bool {
            ps.iter().any(|p| p.is_rest())
        }

        match &pat.kind {
            PatKind::Range(.., Spanned { node: RangeEnd::Included(RangeSyntax::DotDotEq), .. }) => {
                self.record_lang_feature(sym::dotdoteq_in_patterns, pat.span);
            },
            PatKind::Tuple(ps) if has_rest(ps) => {
                self.record_lang_feature(sym::dotdot_in_tuple_patterns, pat.span);
            },
            PatKind::TupleStruct(_, ps) if ps.len() > 1 && has_rest(ps) => {
                self.record_lang_feature(sym::dotdot_in_tuple_patterns, pat.span);
            },
            _ => {},
        }

        visit::walk_pat(self, pat);
    }

    fn visit_path_segment(&mut self, span: Span, segment: &ast::PathSegment) {
        if segment.ident.name == kw::Crate {
            self.record_lang_feature(sym::crate_in_paths, segment.ident.span);
        }

        visit::walk_path_segment(self, span, segment);
    }

    fn visit_mac(&mut self, _mac: &ast::Mac) {
        // Do nothing.
    }
}

impl Visitor {
    fn record_lang_feature(&mut self, feature: Symbol, span: Span) {
        self.lang_features.entry(feature).or_default().insert(span);
    }
}

pub fn walk_crate(wrapper: &mut Wrapper, krate: &ast::Crate, source_map: &SourceMap) {
    let mut visitor = Visitor::default();
    visit::walk_crate(&mut visitor, &krate);

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

// Ideally we would not resolve the macros again, but after the definition is loaded the macro
// is expanded and we don't have access to the stability attributes anymore. Here we collect
// imported macro names and stability attributes and later in the HIR pass we relate the features
// to the spans that come from macro expansion.
// Shouldn't we have MacroDefs available later when visiting the HIR?
pub fn process_imported_macros(session: &Session, resolver: &mut Resolver) -> HashMap<String, Stability> {
    session
        .imported_macro_spans
        .borrow()
        .iter()
        .filter_map(|(_, (name, _))| {
            let path = ast::Path::from_ident(ast::Ident::from_str(name));
            match resolver.resolve_macro_path(&path, None, &ParentScope::module(resolver.graph_root()), false, false) {
                Ok((Some(ext), ..)) => match ext.stability {
                    Some(stab @ Stability { level: StabilityLevel::Stable { .. }, .. }) => Some((name.clone(), stab)),
                    _ => None,
                },
                _ => None,
            }
        })
        .collect()
}