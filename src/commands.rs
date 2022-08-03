use std::path::PathBuf;

use anyhow::Result;

use crate::data_file;

/// Parse a CSV file and write its contents to a Parquet file,
/// replacing .csv extension by .parquet.
pub fn csv_to_parquet(csv_path: &PathBuf, debug: bool) -> Result<()> {
    if debug {
        println!("DEBUG: convert CSV file {:?}", csv_path)
    }
    let mut dataframe = data_file::load_csv_file(csv_path)?;
    let parquet_path = csv_path.with_extension("parquet");
    if debug {
        println!("DEBUG: Writing dataframe to file {:?}", parquet_path)
    }
    data_file::save_parquet_file(&parquet_path, &mut dataframe)?;

    Ok(())
}

/// Parse a Parquet file and write its contents to a CSV file,
/// replacing .parquet extension by .csv.
pub fn parquet_to_csv(parquet_path: &PathBuf, debug: bool) -> Result<()> {
    if debug {
        println!("DEBUG: convert Parquet file {:?}", parquet_path)
    }
    let mut dataframe = data_file::load_parquet_file(parquet_path)?;
    let csv_path = parquet_path.with_extension("csv");
    if debug {
        println!("DEBUG: Writing dataframe to file {:?}", csv_path)
    }
    data_file::save_csv_file(&csv_path, &mut dataframe)?;

    Ok(())
}

pub fn csv_schema(csv_path: &PathBuf) -> Result<String> {
    let lazy_df = data_file::scan_csv_file(csv_path)?;
    Ok(format!("{:?}", lazy_df.schema()))
}

pub fn parquet_schema(parquet_path: &PathBuf) -> Result<String> {
    let lazy_df = data_file::scan_parquet_file(parquet_path)?;
    Ok(format!("{:?}", lazy_df.schema()))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::fs;

    use polars::prelude::DataType;

    use super::*;

    #[test]
    fn csv_schema_fails_on_non_existent_file() {
        let non_existent_path = PathBuf::from(OsStr::new("this_path_does_not_exist"));
        let result = csv_schema(&non_existent_path);
        assert!(result.is_err());

        let error_txt = result.err().unwrap().to_string();
        assert_eq!(
            error_txt,
            "could not scan CSV file \"this_path_does_not_exist\""
        );
    }

    #[test]
    fn convert_fails_on_non_existent_file() {
        let non_existent_path = PathBuf::from(OsStr::new("this_path_does_not_exist"));
        let result = csv_to_parquet(&non_existent_path, false);
        assert!(result.is_err());

        let error_txt = result.err().unwrap().to_string();
        assert_eq!(
            error_txt,
            "could not open CSV file \"this_path_does_not_exist\""
        );
    }

    #[test]
    fn csv_schema_works_on_valid_file() {
        let csv_path = PathBuf::from(OsStr::new("test_data.csv"));
        let result = csv_schema(&csv_path);
        assert!(result.is_ok());

        let schema = result.unwrap();
        let correct_schema = [
            "Schema:",
            "name: Text, data type: Utf8",
            "name: Number, data type: Int64",
            "name: Flag, data type: Boolean",
            "name: Date, data type: Date",
        ]
        .join("\n");
        assert_eq!(schema.trim(), correct_schema);
    }

    #[test]
    fn convert_works_on_valid_file() {
        let csv_path = PathBuf::from(OsStr::new("test_data.csv"));
        let result = csv_to_parquet(&csv_path, false);
        assert!(result.is_ok());

        let parquet_path = csv_path.with_extension("parquet");
        assert!(parquet_path.is_file());

        let data_frame = data_file::load_parquet_file(&parquet_path).unwrap();
        assert_eq!(data_frame.shape(), (3, 4));
        assert_eq!(
            data_frame.get_column_names(),
            &["Text", "Number", "Flag", "Date"]
        );
        assert_eq!(
            data_frame.dtypes(),
            &[
                DataType::Utf8,
                DataType::Int64,
                DataType::Boolean,
                DataType::Date
            ]
        );
        assert!(fs::remove_file(parquet_path).is_ok());
    }
}
