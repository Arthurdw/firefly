/// Create, Read and Delete records example
use ffly_rs::FireflyStream;

static FIREFLY_ADDR: &'static str = "127.0.0.1:46600";

#[tokio::main]
async fn main() {
    let firefly = FireflyStream::connect(FIREFLY_ADDR)
        .await
        .expect("Could not connect to Firefly server!");
    println!("Connected to Firefly server!");

    firefly
        .new("key", "value")
        .await
        .expect("Could not create a new record!");
    println!("Successfully created a new record!");

    assert_eq!(
        firefly
            .get_value("key")
            .await
            .expect("Could not get value!"),
        "value"
    );
    println!("Successfully fetched a record!");

    firefly.drop("key").await.expect("Could not drop a record!");
}
