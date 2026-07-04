use calamine::{DataType, Error, RangeDeserializerBuilder, Reader, Xlsx, open_workbook};

pub fn readXLSSeqs(path: String) -> Option<Vec<Vec<u32>>> {
    // let path = format!("{}/tests/temperature.xlsx", env!("CARGO_MANIFEST_DIR"));
    let mut results: Vec<Vec<u32>> = Vec::new();
    let mut currentseq: Vec<u32> = Vec::new();
    let mut excel: Xlsx<_> = open_workbook(path).unwrap();
    if let Ok(r) = excel.worksheet_range("SPREIDING TV FILM") {
        for row in r.rows() {
            if let Some(x) = row[1].as_i64() {
                if x <= 25 {
                    currentseq.push(x as u32);
                }
            } else {
                results.push(currentseq.clone());
                currentseq.clear();
            }
            println!("row={:?}, row[0]={:?}", row, row[0]);
        }
        results.push(currentseq.clone());
        return Some(results);
    } else {
        return None;
    }
}
