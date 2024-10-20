use crate::constants::PAGE_SIZE_IN_BYTES;

/*   Page structure
        1. PAGE TYPE [u8]
        2. PAGE DATA(dependent of type of page)
*/


/*  PAGE TYPE: [u8] 
       1. Heap (data) page:  0
       2. Internal page:     1
       3. Leaf page:         2
*/

/* data page structure 
        number_of_items: u16  [0..1]
        data_start_offset: u16 [2..3]
        page_heap_top: u16 [4..5] - indicates start of free space
        page_free: u16 [6..7]- indicates end of free space
        data: vec<u8> [data_start_offset..data_end_offset]
        free_space: vec<u8>
        offsets_of_specific_item: vec<u16> [data_end_offset..data_end_offset + number_of_items * size(u16)]
*/ 

pub struct NewPage {
    pub data: [u8; PAGE_SIZE_IN_BYTES as usize],
}

impl NewPage {
    pub fn new(data: [u8; PAGE_SIZE_IN_BYTES as usize]) -> Self {
        NewPage { data }
    }

    pub fn get_header_size_in_bytes(&self) -> u8 {
        return 1;
    }

    pub fn get_first_tuple(&self) -> Result<&[u8], String> {
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
        if first_entry_start_offset > second_entry_start_offset
            || second_entry_start_offset as usize > self.data.len()
        {
            return Err("Invalid offsets".to_string());
        }
        let first_entry =
            &self.data[first_entry_start_offset as usize..second_entry_start_offset as usize];
        Ok(first_entry)
    }

    pub fn get_tuple(&self, start_offset: u16) -> Result<&[u8], String> {
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
        Ok(&self.data[start_offset as usize..end_offset as usize])
    }

    fn get_number_of_items(&self) -> u16 {
        return u16::from_le_bytes(self.data[0..1].try_into().unwrap());
    }

    fn get_page_heap_top(&self) -> u16 {
        return u16::from_le_bytes(self.data[4..5].try_into().unwrap());
    }

    fn get_line_pointer(&self, index: u16) -> u16 {
        let line_pointer_offset = PAGE_SIZE_IN_BYTES - 2 - 2 * (index as u32);
        return u16::from_le_bytes(
            self.data[line_pointer_offset as usize..(line_pointer_offset + 2) as usize]
                .try_into()
                .unwrap(),
        );
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
}
