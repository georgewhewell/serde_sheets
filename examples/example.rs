use serde::{Deserialize, Serialize};
use serde_sheets::{get_sheets, service_account_from_env};

const DOCUMENT_ID: &str = "17jj0gGuYCAfML2ZGA9Go493Pdozn2ogZQ0d2P9I6r6A";
const TAB_NAME: &str = "IntegrationTest";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ExampleObject {
    name: String,
    number_of_foos: u64,
    number_of_bars: f64,
}

fn generate_sample_objects(n: u64) -> Vec<ExampleObject> {
    (0..n)
        .map(|i| ExampleObject {
            name: format!("Object {}", i),
            number_of_foos: i * 10,
            number_of_bars: i as f64 + 0.5,
        })
        .collect()
}

#[tokio::main]
async fn main() {
    let service_account = service_account_from_env().unwrap();
    let mut sheets = get_sheets(service_account, Some("token_cache.json"))
        .await
        .unwrap();

    let objects = generate_sample_objects(50);

    // write first 45 rows to sheet
    serde_sheets::write_page(&mut sheets, DOCUMENT_ID, TAB_NAME, &objects[0..45])
        .await
        .unwrap();

    // append last 5 rows
    for obj in &objects[45..50] {
        serde_sheets::append_row(&mut sheets, DOCUMENT_ID, TAB_NAME, obj)
            .await
            .unwrap();
    }

    // fetch all rows
    let returned: Vec<ExampleObject> = serde_sheets::read_all(&mut sheets, DOCUMENT_ID, TAB_NAME)
        .await
        .unwrap();

    // check data is same
    assert_eq!(objects, returned);
}
