use try_blocks::try_blocks;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[try_blocks]
pub fn smoke_free_fn() -> Result<()> {
    try {
        let _ = foo()?;

        let _: Result<_> = try {
            foo()?;
            42
        };

        let _: Result<_> = try {
            foo()?;
        };
    }
}

#[try_blocks]
pub async fn smoke_async_fn() -> Result<()> {
    try {
        let _ = foo()?;

        (async {
            let _: Result<_> = try {
                bar().await?;

                (async {}).await;

                42
            };
        })
        .await;
    }
}

pub struct X;

impl X {
    #[try_blocks]
    pub fn smoke_method(&self) -> Result<()> {
        try {
            let _ = foo()?;

            let _: Result<_> = try {
                foo()?;
                42
            };
        }
    }
}

#[try_blocks]
impl X {
    pub fn smoke_item_impl(&self) -> Result<()> {
        try {
            let _ = foo()?;
        }
    }
}

#[try_blocks]
pub fn try_op_described_outside_of_try_block() -> Result<()> {
    let _ = foo()?;
    Ok(())
}

fn foo() -> Result<i32> {
    Ok(0)
}

async fn bar() -> Result<i32> {
    Ok(0)
}
