use byteorder::{ByteOrder, LittleEndian};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::db_metadata::DbMetadata;

const PAGE_SIZE: usize = 8 * 1024;

struct BuffPool {
    page_fragmes: Vec<Page>,
}

struct Page {
    items: Vec<Vec<Vec<u8>>>,
}

fn load_page(file_name: String, page_number: u16) -> Result<Page, String> {
    let mut file = File::open(file_name).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start((PAGE_SIZE * page_number as usize)));
    let mut page_content = vec![0; PAGE_SIZE];

    file.read(&mut page_content).map_err(|e| e.to_string())?;
    let number_of_items = page_content[0..u16::BITS];
    let items_start_offset = page_content[number_of_items.len()..number_of_items.len() + u16::BITS];
    let items_end_offset = page_content[number_of_items.len() + items_start_offset.len()
        ..number_of_items.len() + items_start_offset.len() + u16::BITS];
    let items_end_offset_parsed = LittleEndian::read_u16(&items_end_offset);
    let items_offsets = &page_content[items_end_offset_parsed..];
    let number_of_items_parrsed = LittleEndian::read_u16(&number_of_items);
    let items_offsets_parrsed = Vec::with_capacity(number_of_items_parrsed);
    for item_offset in items_offsets.chunks(8) {
        let item_offset_parrsed = LittleEndian::read_u16(item_offset);
        items_offsets_parrsed.push(item_offset_parrsed);
    }
    let items = Vec::with_capacity(number_of_items_parrsed);
    let mut i: u16 = 0;
    while i <= number_of_items_parrsed {
        if i != number_of_items_parrsed {
            items.push(&page_content[items_offsets_parrsed[i]..items_offsets_parrsed[i + 1]]);
        } else {
            items.push(&page_content[items_offsets_parrsed[i]..items_end_offset_parsed]);
        }
        i += 1;
    }
    Ok(Page { items })
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::{Seek, SeekFrom, Write};
    use tempfile::tempfile;

    #[test]
    fn when_given_valid_file_page_should_be_correctly_parssed() {
        let mut temp_file = tempfile().expect("Failed to create temporary file");
        let mut test_data = vec![0; PAGE_SIZE];
        let number_of_items: u16 = 2;
        test_data[0..2].copy_from_slice(&number_of_items.to_le_bytes());
        let item_start_offset: u16 = 100;
        let item_end_offset: u16 = 200;
        test_data[2..4].copy_from_slice(&item_start_offset.to_le_bytes());
        test_data[4..6].copy_from_slice(&item_end_offset.to_le_bytes());
        let item1 = vec![1, 2, 3, 4, 5];
        let item2 = vec![6, 7, 8, 9, 10];
        test_data[item_start_offset as usize..item_start_offset as usize + item1.len()]
            .copy_from_slice(&item1);
        test_data[item_end_offset as usize..item_end_offset as usize + item2.len()]
            .copy_from_slice(&item2);
        temp_file
            .write_all(&test_data)
            .expect("Failed to write to temporary file");

        let result = load_page(temp_file.path().to_string_lossy().into_owned(), 0);
    }
}
