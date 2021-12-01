use csv::{ReaderBuilder, StringRecord, Writer, WriterBuilder};
use google_sheets4::{
    api::{ClearValuesRequest, ValueRange},
    Sheets,
};
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use thiserror::Error;
use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};

#[derive(Error, Debug)]
pub enum SheetsError {
    #[error("SERVICE_ACCOUNT_JSON not defined")]
    EnvVarNotFound(#[from] std::env::VarError),

    #[error("Invalid service account JSON")]
    InvalidServiceAccountJSON(#[from] serde_json::Error),

    #[error("Error with token cache path")]
    TokenCachePathError(#[from] std::io::Error),

    #[error(transparent)]
    SheetsError(#[from] google_sheets4::Error),

    #[error(transparent)]
    CSVError(#[from] csv::Error),

    #[error("Internal error")]
    InternalUTFError(#[from] std::string::FromUtf8Error),

    #[error("Internal error")]
    InternalWriterError(#[from] csv::IntoInnerError<Writer<Vec<u8>>>),
}

/// Builds a `ServiceAccountKey` from JSON in environment variable `SERVICE_ACCOUNT_JSON`
pub fn service_account_from_env() -> Result<ServiceAccountKey, SheetsError> {
    let env = std::env::var("SERVICE_ACCOUNT_JSON")?;
    let key = serde_json::from_str(&env)?;
    Ok(key)
}

/// Given a `ServiceAccountKey`, builds a `google_sheets4::Sheets` client, with
/// access token cache at `token_cache_path` (if specified)
pub async fn get_sheets<P: Into<PathBuf>>(
    service_account: ServiceAccountKey,
    token_cache_path: Option<P>,
) -> Result<Sheets, SheetsError> {
    let builder = ServiceAccountAuthenticator::builder(service_account);
    let auth = if let Some(path) = token_cache_path {
        builder.persist_tokens_to_disk(path).build().await?
    } else {
        builder.build().await?
    };
    let sheets = Sheets::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );
    Ok(sheets)
}

/// Clear all data from the sheet called `tab_name` in document `document_id`
pub async fn clear_tab(
    sheets: &mut Sheets,
    document_id: &str,
    tab_name: &str,
) -> Result<(), SheetsError> {
    sheets
        .spreadsheets()
        .values_clear(ClearValuesRequest::default(), document_id, tab_name)
        .doit()
        .await?;
    Ok(())
}

/// Serialize a list of objects and write to the tab `tab_name` in document `document_id`.
/// The sheet will be cleared before writing.
pub async fn write_page(
    sheets: &mut Sheets,
    document_id: &str,
    tab_name: &str,
    objects: &[impl serde::Serialize],
) -> Result<(), SheetsError> {
    clear_tab(sheets, document_id, tab_name).await?;

    let mut wtr = WriterBuilder::new().from_writer(vec![]);

    for obj in objects {
        wtr.serialize(&obj)?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(data.as_bytes());
    let records = rdr
        .records()
        .collect::<Result<Vec<StringRecord>, csv::Error>>()?;

    let req = ValueRange {
        major_dimension: None,
        range: Some(tab_name.to_string()),
        values: Some(
            records
                .into_iter()
                .map(|s| s.iter().map(|s| s.to_string()).collect())
                .collect(),
        ),
    };

    sheets
        .spreadsheets()
        .values_update(req, document_id, tab_name)
        .value_input_option("USER_ENTERED")
        .include_values_in_response(false)
        .doit()
        .await?;

    Ok(())
}

/// Append a single object `obj` to tab `tab_name` in document `document_id`
pub async fn append_row(
    sheets: &mut Sheets,
    document_id: &str,
    tab_name: &str,
    obj: impl serde::Serialize,
) -> Result<(), SheetsError> {
    let mut wtr = WriterBuilder::new().from_writer(vec![]);

    wtr.serialize(&obj)?;

    let data = String::from_utf8(wtr.into_inner()?)?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(data.as_bytes());

    let records = rdr
        .records()
        .collect::<Result<Vec<StringRecord>, csv::Error>>()?;

    let req = ValueRange {
        major_dimension: None,
        range: Some(tab_name.to_string()),
        values: Some(
            records
                .into_iter()
                .map(|s| s.iter().map(|s| s.to_string()).collect())
                .collect(),
        ),
    };

    sheets
        .spreadsheets()
        .values_append(req, document_id, tab_name)
        .value_input_option("USER_ENTERED")
        .include_values_in_response(false)
        .doit()
        .await?;

    Ok(())
}

/// Append a single object `obj` to tab `tab_name` in document `document_id`
pub async fn read_all<T: DeserializeOwned>(
    sheets: &mut Sheets,
    document_id: &str,
    tab_name: &str,
) -> Result<Vec<T>, SheetsError> {
    let (_body, value_range) = sheets
        .spreadsheets()
        .values_get(document_id, tab_name)
        .doit()
        .await?;

    let rows = value_range.values.unwrap();

    let mut wtr = WriterBuilder::new().from_writer(vec![]);

    for row in rows {
        wtr.write_record(&row)?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(data.as_bytes());

    let mut records = vec![];
    for result in rdr.deserialize() {
        let record: T = result?;
        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
