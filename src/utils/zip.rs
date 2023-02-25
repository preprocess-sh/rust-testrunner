use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use zip::{read::ZipFile, ZipArchive};

fn process_zip_string(
    zip_string: String,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut files: HashMap<String, String> = HashMap::new();

    // Decode the base64 ZIP string to an array of bytes
    let decoded_zip_bytes: Vec<u8> =
        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD)
            .decode(zip_string.as_str())
            .map_err(|err| format!("Couldn't decode ZIP string: {}", err))?;

    // Create a ZIP from the decoded bytes
    let decoded_zip: ZipArchive<Cursor<Vec<u8>>> = ZipArchive::new(Cursor::new(decoded_zip_bytes))
        .map_err(|err| format!("Couldn't open ZIP archive: {}", err))?;
    let decoded_zip_length: usize = decoded_zip.len();

    for i in 0..decoded_zip_length {
        let mut decoded_zip = decoded_zip.clone();
        let mut file: ZipFile = decoded_zip
            .by_index(i)
            .map_err(|err| format!("Couldn't open file in ZIP archive: {}", err))?;

        let filename = file.name().to_owned();
        let mut contents: String = String::new();

        file.read_to_string(&mut contents)
            .map_err(|e| format!("Couldn't read file in ZIP archive: {}", e))?;

        files.insert(filename.clone(), contents.clone());
    }

    Ok(files)
}
