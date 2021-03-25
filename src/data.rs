use anyhow::bail;
use csv::Trim;
use std::collections::HashMap;

pub fn parse(input: &[u8]) -> anyhow::Result<HashMap<String, String>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true) // we want the same error emssage on the first record as the rest
        .trim(Trim::All)
        .comment(Some(b'#'))
        .from_reader(input);

    let mut output = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        if record.len() != 2 {
            bail!(
                "Failed to validate data: line {}: expected record with 2 fields, found {} fields",
                record.position().unwrap().line(),
                record.len()
            );
        }
        let key = record.get(0).unwrap().trim();
        let value = record.get(1).unwrap().trim();
        if output.insert(key.to_string(), value.to_string()).is_some() {
            bail!("Duplicate key {}", key);
        }
    }
    Ok(output)
}
