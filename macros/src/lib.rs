use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, quote_spanned};
use syn::{
    fold::{self, Fold}, //
    parse::Parser as _,
    spanned::Spanned,
    Attribute,
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
    let expanded = TryBlocksExpander::default().expand_item(item);
    TokenStream::from(quote!(#expanded))
}

#[proc_macro_hack]
pub fn try_block(input: TokenStream) -> TokenStream {
    let parser = Block::parse_within;
    let block = match parser.parse(input) {
        Ok(stmts) => Block {
            stmts,
            brace_token: Default::default(),
        },
        Err(err) => return err.to_compile_error().into(),
    };
    let expanded = TryBlocksExpander::default().expand_try_block(vec![], block);
    TokenStream::from(quote!(#expanded))
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
    fn expand_item(&mut self, item: Item) -> Item {
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

    fn expand_try_block(&mut self, attrs: Vec<Attribute>, block: Block) -> Expr {
        self.with_try_scope(|me| {
            let block_span = block.span();

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

            Expr::Verbatim(quote_spanned! { block_span => {
                use ::try_blocks::_rt as __try_blocks;
                #(#attrs)*
                #name: loop {
                    #expanded
                }
            }})
        })
    }

    fn expand_try(&mut self, expr: ExprTry) -> Expr {
        let ExprTry {
            attrs,
            expr,
            question_token,
        } = expr;

        let expanded = self.fold_expr(*expr);

        if let Some(scope) = self.try_scopes.last() {
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
        } else {
            Expr::Try(ExprTry {
                attrs,
                expr: Box::new(expanded),
                question_token,
            })
        }
    }
}

impl Fold for TryBlocksExpander {
    fn fold_item(&mut self, item: Item) -> Item {
        item // ignore inner items
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::TryBlock(ExprTryBlock {
                attrs, //
                block,
                ..
            }) => self.expand_try_block(attrs, block),
            Expr::Try(e) => self.expand_try(e),
            expr => fold::fold_expr(self, expr),
        }
    }
}
