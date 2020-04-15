use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    fold::{self, Fold},
    Expr, ExprLoop, ExprTry, ExprTryBlock, Item, ItemFn, Stmt,
};

#[proc_macro_attribute]
pub fn try_blocks(_: TokenStream, item: TokenStream) -> TokenStream {
    let item: ItemFn = syn::parse_macro_input!(item);

    let item = TryBlockExpander { try_scopes: vec![] }.fold_item_fn(item);

    TokenStream::from(quote::quote! {
        #item
    })
}

struct TryBlockExpander {
    try_scopes: Vec<syn::Lifetime>,
}

impl TryBlockExpander {
    fn with_try_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let len = self.try_scopes.len();
        self.try_scopes.push(syn::Lifetime::new(
            &format!("'__try_blocks_{}", len), //
            Span::call_site(),
        ));

        let result = f(self);
        assert_eq!(len + 1, self.try_scopes.len());

        self.try_scopes.pop().unwrap();

        result
    }
}

impl Fold for TryBlockExpander {
    fn fold_item(&mut self, item: Item) -> Item {
        item // ignore inner items
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::TryBlock(ExprTryBlock { attrs, block, .. }) => self.with_try_scope(|me| {
                let mut expanded = me.fold_block(block);
                let name = me.try_scopes.last().unwrap();

                {
                    let last_non_item_stmt = expanded
                        .stmts
                        .iter_mut()
                        .rev()
                        .filter(|stmt| !matches!(stmt, syn::Stmt::Item(..)))
                        .next();

                    match last_non_item_stmt {
                        Some(syn::Stmt::Expr(ref mut expr)) => {
                            *expr = Expr::Verbatim(quote::quote! {
                                break #name ::try_blocks::_reexports::Try::from_ok(#expr)
                            });
                        }
                        _ => {
                            expanded.stmts.push(Stmt::Semi(
                                Expr::Verbatim(quote::quote! {
                                    break #name ::try_blocks::_reexports::Try::from_ok(())
                                }),
                                Default::default(),
                            ));
                        }
                    }
                }

                Expr::Loop(ExprLoop {
                    attrs,
                    label: Some(syn::Label {
                        name: name.clone(),
                        colon_token: Default::default(),
                    }),
                    loop_token: Default::default(),
                    body: expanded,
                })
            }),
            Expr::Try(ExprTry { attrs, expr, .. }) => {
                let expanded = self.fold_expr(*expr);
                let name = self.try_scopes.last().unwrap();
                Expr::Verbatim(quote::quote! {
                    #(#attrs)*
                    match ::try_blocks::_reexports::Try::into_result(#expanded) {
                        #[allow(unreachable_code)]
                        Ok(val) => val,
                        #[allow(unreachable_code)]
                        Err(err) => {
                            break #name ::try_blocks::_reexports::Try::from_error(
                                From::from(err)
                            );
                        },
                    }
                })
            }
            expr => fold::fold_expr(self, expr),
        }
    }
}
