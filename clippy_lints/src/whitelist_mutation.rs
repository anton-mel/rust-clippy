use rustc_lint::{LateContext, LateLintPass};
use rustc_data_structures::fx::FxHashSet;
use rustc_ast::token::{Token, TokenKind};
use rustc_ast::tokenstream::TokenTree;
use rustc_session::impl_lint_pass;
use rustc_ast::{AttrArgs, AttrKind};
use rustc_hir::{
    intravisit,
    intravisit::Visitor,
    HirId, ItemKind};

declare_clippy_lint! {
    /// ### What it does
    /// Checks if a struct field is mutated only by functions specified in the `#[mutatedby(...)]` attribute.
    ///
    /// ### Why restrict this?
    /// To ensure that certain fields are only modified by specific functions to maintain encapsulation and control over field mutations.
    ///
    /// ### Example
    /// ```rust
    /// pub struct MyStruct {
    ///     #[mutatedby("allowed_function")]
    ///     field1: u8,
    /// }
    ///
    /// impl MyStruct {
    ///     fn allowed_function(&mut self) {
    ///         self.field1 = 10;
    ///     }
    ///
    ///     fn disallowed_function(&mut self) {
    ///         self.field1 = 20; // This should trigger a lint warning
    ///     }
    /// }
    /// ```
    #[clippy::version = "1.81.0"]
    pub FIELDS_MUTATED_BY_WHITELIST,
    restriction,
    "ensures that a field is only mutated by functions specified in the #[mutatedby(...)] attribute"
}

impl_lint_pass!(FieldsMutatedByWhitelist => [FIELDS_MUTATED_BY_WHITELIST]);

pub struct FieldsMutatedByWhitelist {
    pub allowed_functions: FxHashSet<String>,
}

impl FieldsMutatedByWhitelist {
    pub fn new() -> Self {
        Self {
            allowed_functions: FxHashSet::default(),
        }
    }

    pub fn add_function(&mut self, function_name: &str) {
        self.allowed_functions.insert(function_name.to_string());
    }
}

impl LateLintPass<'_> for FieldsMutatedByWhitelist {
    fn check_crate(&mut self, cx: &LateContext<'_>) {
        let mut visitor = FieldVisitor {
            cx,
            allowed_functions: &mut self.allowed_functions,
        };
        cx.tcx.hir().visit_all_item_likes_in_crate(&mut visitor);
    }
}

struct FieldVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    allowed_functions: &'a mut FxHashSet<String>,
}

impl<'a, 'tcx> Visitor<'tcx> for FieldVisitor<'a, 'tcx> {
    fn visit_item(&mut self, item: &'tcx rustc_hir::Item<'tcx>) {
    if let ItemKind::Struct(ref _struct, _) = item.kind {
        self.check_struct_fields(item.hir_id());
    }
    intravisit::walk_item(self, item);
}
}

impl<'a, 'tcx> FieldVisitor<'a, 'tcx> {
    fn check_struct_fields(&mut self, struct_hir_id: HirId) {
        let attrs = self.cx.tcx.hir().attrs(struct_hir_id);

        for attr in attrs {
            if let AttrKind::Normal(normal_attr) = &attr.kind {
                // Correct pattern matching for `AttrArgs::Delimited`
                if let AttrArgs::Delimited(delimited) = &normal_attr.item.args {
                    let token_trees = delimited.tokens.trees();

                    // Collect function names from tokens
                    let function_names: Vec<String> = token_trees
                        .filter_map(|tt| match tt {
                            TokenTree::Token(
                                Token {
                                    kind: TokenKind::Ident(ident, _),
                                    ..
                                },
                                _,
                            ) => Some(ident.to_string()),
                            _ => None,
                        })
                        .collect();

                    // Add each function name to the allowed functions
                    for function_name in function_names.clone() {
                        self.allowed_functions.insert(function_name);
                    }
                }
            }
        }
    }
}
