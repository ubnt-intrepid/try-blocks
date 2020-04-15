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

#[derive(Default)]
struct TryBlocksExpander {
    try_scopes: Vec<Lifetime>,
}

impl TryBlocksExpander {
    fn expand(&mut self, item: Item) -> Item {
        fold::fold_item(self, item)
    }

    fn with_try_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let len = self.try_scopes.len();
        self.try_scopes.push(Lifetime::new(
            &format!("'__try_blocks_{}", len), //
            Span::call_site(),
        ));

        let result = f(self);
        assert_eq!(len + 1, self.try_scopes.len());

        self.try_scopes.pop().unwrap();

        result
    }

    fn expand_try_block(&mut self, expr: ExprTryBlock) -> Expr {
        let ExprTryBlock {
            attrs,
            block,
            try_token,
            ..
        } = expr;

        self.with_try_scope(|me| {
            let mut expanded = me.fold_block(block);
            let name = me.try_scopes.last().unwrap();

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
                            Expr::Verbatim(quote_spanned! { try_token.span() =>
                                break #name __try_blocks::from_ok(())
                            }),
                            Default::default(),
                        ));
                    }
                }
            }

            Expr::Verbatim(quote_spanned! { try_token.span() =>
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
        let name = self.try_scopes.last().unwrap();

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
            syn::parse_quote! {
                use ::try_blocks::_rt as __try_blocks;
            },
        );

        expanded
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::TryBlock(e) => self.expand_try_block(e),
            Expr::Try(e) => self.expand_try(e),
            expr => fold::fold_expr(self, expr),
        }
    }
}
