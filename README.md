# serde_sheets

Read and write structs directly from google sheets using `serde` and `csv`

Implement `serde::Serialize` to write and `serde::Deserialize` to read. Easy!

## Motivation

Google sheets API is somewhat complex and requires lots of boilerplate. Reading and 
writing involves using lots of `Vec<Vec<String>>`. We use `csv` crate to automate
creating these payloads and converting them back into your structs

## Usage

serde_sheets expects a google service account json to be available as `SERVICE_ACCOUNT_JSON`:

    $ export SERVICE_ACCOUNT_JSON=$(cat my-service-account.json)

Build `Sheets` object:

    let service_account = service_account_from_env().unwrap();
    let mut sheets = get_sheets(service_account, Some("token_cache.json"))
        .await
        .unwrap();

Write objects:

    serde_sheets::write_page(&mut sheets, "some-document-id", "some-tab-name", &objects)
        .await
        .unwrap();

Read objects:

    let returned: Vec<ExampleObject> = serde_sheets::read_all(&mut sheets, DOCUMENT_ID, TAB_NAME)
        .await
        .unwrap();

Check `examples/example.rs` for full example.

    $ cargo run --example example

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
