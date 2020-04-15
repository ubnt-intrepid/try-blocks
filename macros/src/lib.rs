use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    fold::{self, Fold}, //
    spanned::Spanned,
    Block,
    Expr,
    ExprTry,
    ExprTryBlock,
    Item,
    Lifetime,
    Stmt,
};

#[proc_macro_attribute]
pub fn try_blocks(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as Item);

    let item = TryBlocksExpander::default().expand(item);

    TokenStream::from(quote! {
        #item
    })
}

struct Scope {
    id: usize,
    name: Lifetime,
}

#[derive(Default)]
struct TryBlocksExpander {
    try_scopes: Vec<Scope>,
    next_scope_id: usize,
}

impl TryBlocksExpander {
    fn expand(&mut self, item: Item) -> Item {
        fold::fold_item(self, item)
    }

    fn with_try_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let scope_id = self.next_scope_id;
        self.try_scopes.push(Scope {
            id: scope_id,
            name: Lifetime::new(
                &format!("'__try_blocks_{}", scope_id), //
                Span::call_site(),
            ),
        });
        self.next_scope_id += 1;

        let result = f(self);

        let scope = self.try_scopes.pop().unwrap();
        assert_eq!(scope.id, scope_id);

        result
    }

    fn expand_try_block(&mut self, expr: ExprTryBlock) -> Expr {
        let ExprTryBlock { attrs, block, .. } = expr;
        let block_span = block.span();

        self.with_try_scope(|me| {
            let mut expanded = me.fold_block(block);
            let scope = me.try_scopes.last().unwrap();
            let name = &scope.name;

            {
                let last_non_item_stmt = expanded
                    .stmts
                    .iter_mut()
                    .rev()
                    .filter(|stmt| !matches!(stmt, Stmt::Item(..)))
                    .next();

                match last_non_item_stmt {
                    Some(Stmt::Expr(ref mut expr)) => {
                        *expr = Expr::Verbatim(quote_spanned! { expr.span() =>
                            break #name __try_blocks::from_ok(#expr)
                        });
                    }
                    _ => {
                        expanded.stmts.push(Stmt::Semi(
                            Expr::Verbatim(quote_spanned! { block_span =>
                                break #name __try_blocks::from_ok(())
                            }),
                            Default::default(),
                        ));
                    }
                }
            }

            Expr::Verbatim(quote_spanned! { block_span =>
                #(#attrs)*
                #name: loop {
                    #expanded
                }
            })
        })
    }

    fn expand_try(&mut self, expr: ExprTry) -> Expr {
        let ExprTry {
            attrs,
            expr,
            question_token,
            ..
        } = expr;

        let expanded = self.fold_expr(*expr);
        let scope = self.try_scopes.last().unwrap();
        let name = &scope.name;

        Expr::Verbatim(quote_spanned! { question_token.span() =>
            #(#attrs)*
            match __try_blocks::into_result(#expanded) {
                #[allow(unreachable_code)]
                Ok(val) => val,

                #[allow(unreachable_code)]
                Err(err) => {
                    break #name __try_blocks::from_error(
                        From::from(err)
                    );
                },
            }
        })
    }
}

impl Fold for TryBlocksExpander {
    fn fold_item(&mut self, item: Item) -> Item {
        item // ignore inner items
    }

    fn fold_block(&mut self, block: Block) -> Block {
        let mut expanded = fold::fold_block(self, block);

        expanded.stmts.insert(
            0,
            Stmt::Item(Item::Verbatim(quote! {
                use ::try_blocks::_rt as __try_blocks;
            })),
        );

        expanded
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::TryBlock(e) => self.expand_try_block(e),
            // ignore `<expr>?` written outside of `try { ... }`
            expr @ Expr::Try(..) if self.try_scopes.is_empty() => expr,
            Expr::Try(e) => self.expand_try(e),
            expr => fold::fold_expr(self, expr),
        }
    }
}
