use try_blocks::try_blocks;

#[try_blocks]
pub fn smoke_free_fn() -> Result<(), Box<dyn std::error::Error>> {
    try {
        let _ = foo()?;

        let _: Result<_, Box<dyn std::error::Error>> = try {
            foo()?;
            42
        };

        let _: Result<_, Box<dyn std::error::Error>> = try {
            foo()?;
        };
    }
}

#[try_blocks]
pub async fn smoke_async_fn() -> Result<(), Box<dyn std::error::Error>> {
    try {
        let _ = foo()?;

        (async {
            let _: Result<_, Box<dyn std::error::Error>> = try {
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
    pub fn smoke_method(&self) -> Result<(), Box<dyn std::error::Error>> {
        try {
            let _ = foo()?;

            let _: Result<_, Box<dyn std::error::Error>> = try {
                foo()?;
                42
            };
        }
    }
}

fn foo() -> std::io::Result<i32> {
    Ok(0)
}

async fn bar() -> std::io::Result<i32> {
    Ok(0)
}
