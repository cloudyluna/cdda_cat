#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cdda_cat::run().await
}
