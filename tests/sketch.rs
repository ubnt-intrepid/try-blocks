use try_blocks::try_blocks;

#[try_blocks]
#[test]
#[ignore]
fn smoke_try_block_expansion() -> Result<(), Box<dyn std::error::Error>> {
    try {
        let _ = foo()?;

        let _: Result<_, Box<dyn std::error::Error>> = try {
            bar()?;
            42
        };
    }
}

fn foo() -> std::io::Result<i32> {
    Ok(0)
}

fn bar() -> std::io::Result<i32> {
    Ok(0)
}
