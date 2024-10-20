/*
    File defintion:
        number_of_items: u16  [0..1]
        data_start_offset: u16 [2..3]
        page_heap_top: u16 [4..5] - indicates start of free space
        page_free: u16 [6..7]- indicates end of free space
        data: vec<u8> [data_start_offset..data_end_offset]
        free_space: vec<u8>
        offsets_of_specific_item: vec<u16> [data_end_offset..data_end_offset + number_of_items * size(u16)]
*/

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const PAGE_SIZE: u16 = 8 * 1024;

pub struct Page {
    pub from: String,
    pub page_number: u32,
    pub data: Vec<u8>,
}

impl Page {

    pub fn get_first_tuple_from_page(&self) -> Result<Vec<u8>, String> {
        let first_entry_start_offset = {
            let serialized = self.data[self.data.len() - 4..].to_vec();
            u32::from_le_bytes(
                serialized
                    .try_into()
                    .map_err(|_| "Slice with incorrect length".to_string())?,
            )
        };
        let second_entry_start_offset = {
            let serialized = &self.data[self.data.len() - 8..self.data.len() - 4];
            u32::from_le_bytes(
                serialized
                    .try_into()
                    .map_err(|_| "Slice with incorrect length".to_string())?,
            )
        };
        let first_entry =
            self.data[first_entry_start_offset as usize..second_entry_start_offset as usize].to_vec();
        Ok(first_entry)
    }

    fn insert_tuple(&mut self, serialized_data: &Vec<u8>, offset: u16) {
        let end = offset as usize + serialized_data.len();
        if end > self.data.len() {
            panic!("Data does not fit into the page at the given offset!");
        }
        self.data[offset as usize..end].copy_from_slice(&serialized_data[..]);
        self.set_number_of_items(self.get_number_of_items() + 1);
        self.set_page_heap_top(end.try_into().unwrap());
        self.insert_new_line_pointer(offset);
        self.set_page_free(self.get_page_free() - 2);
    }

    pub fn get_new_insert_pos(&self, bytes_required: u16) -> Option<u16> {
        let data_start_offset = u16::from_le_bytes(self.data[4..5].try_into().unwrap());
        let page_heap_top = u16::from_le_bytes(self.data[6..7].try_into().unwrap());
        let free_bytes = page_heap_top - data_start_offset - 2;
        if bytes_required > free_bytes {
            return None;
        } else {
            return Some(data_start_offset);
        }
    }

    fn get_page_heap_top(&self) -> u16 {
        return u16::from_le_bytes(self.data[4..5].try_into().unwrap());
    }

    fn set_page_heap_top(&mut self, new_heap_top: u16) {
        let new_heap_top_serilized: [u8; 2] = [(new_heap_top >> 8) as u8, (new_heap_top & 0xFF) as u8];
        self.data[4..5].copy_from_slice(&new_heap_top_serilized)
    }

    fn get_page_free(&self) -> u16 {
        return u16::from_le_bytes(self.data[6..7].try_into().unwrap());
    }

    fn set_page_free(&mut self, new_page_free: u16) {
        let new_heap_top_serilized: [u8; 2] = [(new_page_free >> 8) as u8, (new_page_free & 0xFF) as u8];
        self.data[6..7].copy_from_slice(&new_heap_top_serilized)
    }

    fn insert_new_line_pointer(&mut self, offset: u16) {
        let new_line_pointer_serilized = [(offset >> 8) as u8, (offset & 0xFF) as u8];
        let new_line_pointer_end = self.get_page_free() as usize;
        let new_line_pointer_start = (self.get_page_free() - 2) as usize;
        self.data[new_line_pointer_start..new_line_pointer_end].copy_from_slice(&new_line_pointer_serilized)
    }


    fn get_number_of_items(&self) -> u16 {
        return u16::from_le_bytes(self.data[0..1].try_into().unwrap());
    }

    fn set_number_of_items(&mut self, number_od_items: u16) {
        let new_number_od_items_serilized = [(number_od_items >> 8) as u8, (number_od_items & 0xFF) as u8];
        self.data[0..1].copy_from_slice(&new_number_od_items_serilized)
    }

    pub fn get_tuple(&self, start_offset: u16) -> Result<Vec<u8>, String> {
        let mut low = 0;
        let mut high = self.get_number_of_items() - 1;
        let end_offset = loop {
            let i = (low + high) / 2;
            let line_pointer = self.get_line_pointer(i);
            if line_pointer == start_offset {
                if i == self.get_number_of_items() - 1 {
                    break Ok(self.get_page_heap_top());
                } else {
                    break Ok(self.get_line_pointer(i + 1));
                }
            }
            if line_pointer < start_offset {
                low = i + 1;
            } else {
                high = i - 1;
            }
            if low > high {
                break Err("Can't find start offset in line pointer directory".to_string());
            }
        }?;
        if (start_offset as usize) >= self.data.len() || (end_offset as usize) > self.data.len() {
            return Err("Invalid offset range in page data".into());
        }
        Ok(self.data[start_offset as usize..end_offset as usize].to_vec())
    }

    fn get_line_pointer(&self, index: u16) -> u16 {
        let line_pointer_offset =  PAGE_SIZE - 2 - 2 * index;
        return u16::from_le_bytes(self.data[line_pointer_offset as usize..(line_pointer_offset + 2) as usize].try_into().unwrap());
    }
}

fn load_page(file_name: String, page_number: u32) -> Result<Page, String> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_name)
        .map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start((PAGE_SIZE as u32 * page_number).into()))
        .unwrap();
    let mut page_content = vec![0; PAGE_SIZE.try_into().unwrap()];
    file.read(&mut page_content).map_err(|e| e.to_string())?;
    let number_of_items_as_bytes = &page_content[0..2];
    let page_heap_top_offset_as_bytes = &page_content[4..6];
    let page_heap_top = LittleEndian::read_u16(page_heap_top_offset_as_bytes) as usize;
    let page_free_as_bytes = &page_content[6..8];
    let page_free = LittleEndian::read_u16(page_free_as_bytes) as usize;
    let items_offsets_as_bytes = &page_content[page_free..];
    let number_of_items = LittleEndian::read_u16(&number_of_items_as_bytes) as usize;
    let reversed_items_offsets = {
        let mut offsets = Vec::with_capacity(number_of_items);
        for item_offset in items_offsets_as_bytes.chunks(2) {
            let item_offset = LittleEndian::read_u16(item_offset) as usize;
            offsets.push(item_offset);
        }
        offsets
    };
    let rows_data = {
        let mut rows: Vec<Vec<u8>> = Vec::with_capacity(number_of_items);
        let mut i: usize = 0;
        while i < number_of_items {
            let row_data = if i != number_of_items - 1 {
                page_content[reversed_items_offsets[reversed_items_offsets.len() - 1 - i]
                    ..reversed_items_offsets[reversed_items_offsets.len() - 2 - i]]
                    .to_vec()
            } else {
                page_content[reversed_items_offsets.first().unwrap().clone()..page_heap_top]
                    .to_vec()
            };
            rows.push(row_data);
            i += 1;
        }
        rows
    };
    Ok(Page {
        from: file_name.to_string(),
        page_number,
        data: rows_data.into_iter().flatten().collect(),
    })
}

/*fn update_page(file_name: String, page_number: u32, page: &Page) -> Result<(), String> {
     let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)
        .map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start((PAGE_SIZE as u32 * page_number).into()))
        .unwrap();
    let number_of_items = page.rows_data.len();
    let number_of_items_as_bytes = {
        let number_of_items_u16: u16 = number_of_items
            .try_into()
            .map_err(|_| "number_of_items is too large for u16")?;
        let mut buffer = Vec::with_capacity(2);
        buffer
            .write_u16::<LittleEndian>(number_of_items_u16)
            .map_err(|e| e.to_string())?;
        buffer
    };
    let data_start_offset: usize = 8;
    let data_start_offset_as_bytes = {
        let data_start_offset_u16: u16 = data_start_offset
            .try_into()
            .map_err(|_| "Error while prepering header values")?;
        let mut buffer = Vec::with_capacity(2);
        buffer
            .write_u16::<LittleEndian>(data_start_offset_u16)
            .map_err(|e| e.to_string())?;
        buffer
    };
    let page_heap_top: usize =
        data_start_offset + page.rows_data.iter().map(|row| row.len()).sum::<usize>();
    let page_heap_top_as_bytes = {
        let page_heap_top_u16: u16 = page_heap_top
            .try_into()
            .map_err(|_| "Error while prepering header values")?;
        let mut buffer = Vec::with_capacity(2);
        buffer
            .write_u16::<LittleEndian>(page_heap_top_u16)
            .map_err(|e| e.to_string())?;
        buffer
    };
    let page_free = PAGE_SIZE as usize - number_of_items as usize * std::mem::size_of::<u16>();
    let offsets = {
        let mut offsets = Vec::with_capacity(number_of_items);
        let mut item_offset: usize = data_start_offset;
        offsets.push(item_offset);
        for (index, row) in page.rows_data.iter().enumerate() {
            if index < page.rows_data.len() - 1 {
                item_offset += row.len();
                offsets.push(item_offset);
            } else {
                continue;
            }
        }
        offsets.reverse();
        offsets
    };
    let distance_from_data_to_offsets: i64 = (page_free - page_heap_top)
        .try_into()
        .map_err(|_| "Error while converting value to i64")?;
    file.write_all(&number_of_items_as_bytes)
        .map_err(|e| format!("Error writing header: {}", e))?;
    file.write_all(&data_start_offset_as_bytes)
        .map_err(|_| "Error writing header")?;
    file.write_all(&page_heap_top_as_bytes)
        .map_err(|_| "Error writing header")?;
    file.write_all(&((page_free as u16).to_le_bytes()))
        .map_err(|_| "Error writing header")?;
    for row in &page.rows_data {
        file.write_all(&row).map_err(|_| "Error writing data")?;
    }
    file.seek(SeekFrom::Current(distance_from_data_to_offsets))
        .map_err(|_| "Error writing to file")?;
    for item_offset in offsets {
        let item_offset_u16: u16 = item_offset
            .try_into()
            .map_err(|_| "Unexpected value for item offset {e}")?;
        file.write(&item_offset_u16.to_le_bytes())
            .map_err(|_| "Error writing item offset")?;
    }
    Ok(())
}
    */

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, os::unix::fs::FileExt};

    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use tempfile::tempdir;

    #[test]
    fn test_load_page_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.sl");
        let mut file = File::create(&file_path).unwrap();
        let page_number = 2;
        let number_of_items: u16 = 2;
        let data_start_offset: u16 = 8;
        let page_heap_top: u16 = 28;
        let free_page: u16 = 1024 * 8 - 4;
        let item1: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let item2: Vec<u8> = vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        file.seek(SeekFrom::Start(
            (PAGE_SIZE * page_number).try_into().unwrap(),
        ))
        .unwrap();
        file.write_all(&number_of_items.to_le_bytes()).unwrap();
        file.write_all(&data_start_offset.to_le_bytes()).unwrap();
        file.write_all(&page_heap_top.to_le_bytes()).unwrap();
        file.write_all(&free_page.to_le_bytes()).unwrap();
        file.write_all(&item1).unwrap();
        file.write_all(&item2).unwrap();
        let items_offsets: Vec<u16> =
            vec![data_start_offset + item1.len() as u16, data_start_offset];
        let mut items_offsets_as_bytes = Vec::with_capacity(items_offsets.len() * 2);
        for &value in &items_offsets {
            items_offsets_as_bytes
                .write_u16::<LittleEndian>(value)
                .expect("Failed to write u16");
        }
        file.write_all_at(
            &items_offsets_as_bytes,
            (PAGE_SIZE * page_number + free_page).into(),
        )
        .unwrap();

        let result =
            load_page(file_path.to_string_lossy().to_string(), page_number.into()).unwrap();

        /*         assert_eq!(result.rows_data.len(), 2);
        assert_eq!(result.rows_data[0], item1);
        assert_eq!(result.rows_data[1], item2); */
    }

    /* #[test]
    fn test_update_page() {
        let dir = tempdir().unwrap();
        let temp_file_path = dir.path().join("test_page.bin");
        File::create(&temp_file_path).unwrap();
        let page = Page {
            rows_data: vec![vec![1, 2, 3, 4], vec![5, 6, 7]],
        };
        let page_number = 2;
        let expected_number_of_items: u16 = 2;
        let expected_data_start_offset: u16 = 8;
        let expected_page_heap_top: u16 = 8 + 4 + 3;
        let expected_page_free: u16 = PAGE_SIZE - 2 * std::mem::size_of::<u16>() as u16;
        let expected_data: Vec<u8> = page
            .rows_data
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .collect();
        let zero_fill_count: usize = (expected_page_free - expected_page_heap_top) as usize;
        let expected_offsets = vec![(12 as u16).to_le_bytes(), (8 as u16).to_le_bytes()].concat();

        update_page(
            temp_file_path.to_str().unwrap().to_string(),
            page_number,
            &page,
        )
        .unwrap();
        let mut file = OpenOptions::new().read(true).open(&temp_file_path).unwrap();
        let mut buffer = Vec::new();
        file.seek(SeekFrom::Start((page_number * PAGE_SIZE as u32).into()))
            .unwrap();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(
            &buffer[0..2],
            expected_number_of_items.to_le_bytes(),
            "Number of items writen to file not as expected"
        );
        assert_eq!(
            &buffer[2..4],
            expected_data_start_offset.to_le_bytes(),
            "Data start offset writen to file not as expected"
        );
        assert_eq!(
            &buffer[4..6],
            expected_page_heap_top.to_le_bytes(),
            "Page heap top writen to file not as expected"
        );
        assert_eq!(
            &buffer[6..8],
            expected_page_free.to_le_bytes(),
            "Page free writen to file not as expected"
        );
        assert_eq!(
            &buffer[8..15],
            expected_data,
            "Data written to file not as expected"
        );
        assert_eq!(
            &buffer[expected_page_heap_top as usize..expected_page_free as usize],
            vec![0u8; zero_fill_count],
            "Expected free space used"
        );
        assert_eq!(
            &buffer[expected_page_free as usize..],
            expected_offsets,
            "Offsets written to file not as expected"
        );
    } */
}
