use rustc_ast::{ast, visit};
use rustc_parse::{self, MACRO_ARGUMENTS};
use rustc_session::{parse::ParseSess, Session};
use rustc_span::symbol::{sym, Symbol};

use super::{context::StabCtxt, Wrapper};

// NOTE: This visitor is intended to be used only to catch active attributes before they are removed,
// but the approach is not valid as it won't catch attributes generated as a result of macro expansion.
// Another solution is needed.

struct Visitor<'a, 'scx> {
    stab_ctx: &'a mut StabCtxt<'scx>,
    parse_sess: &'a ParseSess,
    // NOTE: sym::target_vendor does not exist
    target_vendor: Symbol,
}

impl<'a, 'scx> Visitor<'a, 'scx> {
    fn new(stab_ctx: &'a mut StabCtxt<'scx>, parse_sess: &'a ParseSess) -> Self {
        Self { stab_ctx, parse_sess, target_vendor: Symbol::intern("target_vendor") }
    }

    fn walk_cfg_metas(&mut self, item: &ast::MetaItem) {
        match &item.kind {
            ast::MetaItemKind::List(items) => {
                if items.len() != 2 && item.name_or_empty() == sym::cfg_attr {
                    self.stab_ctx.record_lang_feature(sym::cfg_attr_multi, item.span);
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
            self.stab_ctx.record_lang_feature(feature, item.span);
        }
    }
}

impl<'ast> visit::Visitor<'ast> for Visitor<'_, '_> {
    fn visit_attribute(&mut self, attr: &ast::Attribute) {
        if attr.has_name(sym::cfg) || attr.has_name(sym::cfg_attr) {
            if let Some(ref item) = attr.meta() {
                self.walk_cfg_metas(item);
            }
        }

        visit::walk_attribute(self, attr);
    }

    fn visit_mac(&mut self, mac: &ast::MacCall) {
        let segments = &mac.path.segments;
        if segments.len() == 1 && segments[0].ident.name == sym::cfg {
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
            self.stab_ctx.record_lang_feature(sym::param_attrs, param.span);
        }

        visit::walk_param(self, param);
    }

    fn visit_generic_param(&mut self, param: &ast::GenericParam) {
        if !param.attrs.is_empty() {
            self.stab_ctx.record_lang_feature(sym::generic_param_attrs, param.attrs[0].span);
        }

        visit::walk_generic_param(self, param);
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        if let ast::ExprKind::Struct(_, fields, _) = &expr.kind {
            for field in fields {
                if !field.attrs.is_empty() {
                    self.stab_ctx.record_lang_feature(sym::struct_field_attributes, field.span);
                }
            }
        }

        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &ast::Pat) {
        if let ast::PatKind::Struct(_, fields, _) = &pat.kind {
            for field in fields {
                if !field.attrs.is_empty() {
                    self.stab_ctx.record_lang_feature(sym::struct_field_attributes, field.span);
                }
            }
        }

        visit::walk_pat(self, pat);
    }
}

pub fn process_crate(wrapper: &mut Wrapper, session: &Session, krate: &ast::Crate) {
    let mut stab_ctx = StabCtxt::new(session);
    let mut visitor = Visitor::new(&mut stab_ctx, &session.parse_sess);
    visit::walk_crate(&mut visitor, &krate);

    stab_ctx.dump(wrapper);
}
