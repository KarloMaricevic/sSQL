use crate::{
    information_schema::{SData, SType},
    new_page::NewPage,
    page::{self, Page},
};
use byteorder::{ByteOrder, LittleEndian};
use core::{fmt::Debug, num};

pub struct NewLeafNode {
    pub keys: Vec<SData>,
    pub values: Vec<TuplePointer>,
}

pub struct NewInnerNode {
    keys: Vec<SData>,
    pointers: Vec<TuplePointer>,
}

#[derive(Clone)]
pub struct TuplePointer {
    pub page: u32,
    pub offset: u16,
}

/*
    node file structure:

    internal:
        number_of_keys: u32 = 0x01,
        keys_and_left_pointers: vec[(key, TuplePointer)],
        last_pointer: TuplePointer,

    leaf:
        number_of_entries: u32 = 0x01,
        keys: vec[some_type],
        values: vec[TuplePointer],
*/

/*  moras dobiti page gdje ces upisati
 posotoje 3 metode: create/update/delete

create:
    kreirat ce na kraju paga i dodati entry u dic
    next_record_pointer: racuna se tako da se zna gdje je kraj novog tupla


update:
    updejta page, ako ima var key, ako stane update, ako ne izbrises ovog i stavisnovi na kraj mape
    next_record_pointer: izracunat
*/

impl NewPage {
    fn get_leaf_node_pointer(
        page: &NewPage,
        key: &SData,
        key_type: &SType,
    ) -> Result<TuplePointer, String> {
        if !page.is_page_leaf_type() {
            return Err("Page is not leaf".into());
        }
        match key {
            SData::INT(key_value) => {
                if key_type != &SType::INT {
                    return Err("Key type doesn't match with type of given key".into());
                }
                let mut slice_to_parse = &page.data[..];
                slice_to_parse = &slice_to_parse[page.get_header_size_in_bytes() as usize..];
                let number_of_keys = u32::from_le_bytes(
                    slice_to_parse[0..4]
                        .try_into()
                        .map_err(|_| "Invalid slice length".to_string())?,
                );
                slice_to_parse = &slice_to_parse[4..];
                let mut left = 0;
                let mut right = number_of_keys;
                while left <= right {                  
                    let mid = (left + right) / 2;
                    let key_position = (mid as usize) * std::mem::size_of::<i32>();
                    let mid_key: i32 = i32::from_le_bytes(slice_to_parse[key_position..key_position + 4].try_into().unwrap());
                    if mid_key == *key_value {
                        let pointer_start = (number_of_keys as usize * std::mem::size_of::<i32>()) + mid_key; // Offset for values
                        let mid_value_position = values_start + (mid as usize * std::mem::size_of::<TuplePointer>());
                        let found_value: TuplePointer = TuplePointer(u32::from_le_bytes(slice_to_parse[mid_value_position..mid_value_position + 4].try_into().unwrap()));
                        return Ok(found_value);
                    } else if mid_key < *search_key {
                        left = mid + 1;
                    } else {
                        right = mid - 1;
                    }
                }
                let last_key_position = (right as usize) * std::mem::size_of::<i32>();
                let last_key: i32 = i32::from_le_bytes(slice_to_parse[last_key_position..last_key_position + 4].try_into().unwrap());
                println!("Key not found. Last key before the search key was: {}", last_key);
                return Err("".into());
            }
            SData::STRING(_) => todo!(),
        }
    }

    fn is_page_leaf_type(&self) -> bool {
        return !self.data.is_empty() && self.data[0] == 0x02;
    }
}

impl NewLeafNode {
    pub fn buffer_fits_type(buffer: &mut &[u8]) -> bool {
        !buffer.is_empty() && buffer[0] == 0x01
    }

    fn write_new_node_to_page(&self, page: Page) -> Result<Page, String> {
        if self.keys.is_empty() {
            return Err("No keys in leaf".to_string());
        }
        if self.values.is_empty() {
            return Err("No values in leaf".to_string());
        }
        let serilized = self.serialize()?;
        let insert_pos = page.get_new_insert_pos(serilized.len().try_into().unwrap());
        Ok(page)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        if self.keys.is_empty() {
            return Err("No keys in leaf".to_string());
        }
        let type_of_node: u8 = 0x01;
        let key_type: u8 = match self.keys.first().unwrap() {
            SData::INT(_) => 0x00,
            SData::STRING(_) => 0x01,
        };
        let number_of_entries = self.keys.len() as u32;
        let serialized_keys_size: u32 = self.keys.iter().map(|key| key.serialized_size()).sum();
        let serialized_values_size = (self.values.len() * 8) as u32;
        let mut serialized_leaf = Vec::with_capacity(
            (1 + 1 + 4 + serialized_keys_size + serialized_values_size)
                .try_into()
                .unwrap(),
        );
        serialized_leaf.push(type_of_node);
        serialized_leaf.push(key_type);
        serialized_leaf.extend_from_slice(&number_of_entries.to_le_bytes());
        for key in &self.keys {
            let mut serialized_key = Vec::with_capacity(key.serialized_size() as usize);
            key.serialize(&mut serialized_key);
            serialized_leaf.extend_from_slice(&serialized_key);
        }
        for pointer in &self.values {
            serialized_leaf.extend_from_slice(&pointer.page.to_le_bytes());
            serialized_leaf.extend_from_slice(&pointer.offset.to_le_bytes());
        }
        Ok(serialized_leaf)
    }

    pub fn deserialize(buffer: &mut &[u8]) -> Result<NewLeafNode, String> {
        let type_of_node: u8 = buffer[0];
        if type_of_node != 0 {
            return Err("Given buffer dosent contain leaf node".to_string());
        }
        let key_type: u8 = buffer[0];
        let number_of_enteries = {
            let number_of_keys_serilized: &[u8] = &buffer[1..3];
            LittleEndian::read_u16(&buffer)
        };
        *buffer = &buffer[1..];
        let keys = {
            let mut keys_deserilized = Vec::with_capacity(number_of_enteries.try_into().unwrap());
            match key_type {
                0x00 => {
                    let value = i32::from_le_bytes(
                        buffer[0..4]
                            .try_into()
                            .map_err(|_| "Slice with incorrect length".to_string())?,
                    );
                    *buffer = &buffer[4..];
                    keys_deserilized.push(SData::INT(value));
                }
                0x01 => {
                    let len = u32::from_le_bytes(
                        buffer[0..4]
                            .try_into()
                            .map_err(|_| "Slice with incorrect length".to_string())?,
                    );
                    *buffer = &buffer[4..];
                    let str_value = String::from_utf8_lossy(&buffer[0..len as usize]).to_string();
                    *buffer = &buffer[len as usize..];
                    keys_deserilized.push(SData::STRING(str_value));
                }
                _ => Err("Unknown key type".to_string())?,
            }
            Ok::<Vec<SData>, String>(keys_deserilized)
        }?;
        let pointers = {
            let mut pointers_deserilized = Vec::with_capacity(number_of_enteries.into());
            for _ in 0..number_of_enteries {
                let page = u32::from_le_bytes(
                    buffer[0..4]
                        .try_into()
                        .map_err(|_| "Slice with incorrect length".to_string())?,
                );
                *buffer = &buffer[4..];
                let offset = u16::from_le_bytes(
                    buffer[0..2]
                        .try_into()
                        .map_err(|_| "Slice with incorrect length".to_string())?,
                );
                *buffer = &buffer[4..];
                pointers_deserilized.push(TuplePointer { page, offset });
            }
            Ok::<Vec<TuplePointer>, String>(pointers_deserilized)
        }?;
        Ok(NewLeafNode {
            keys,
            values: pointers,
        })
    }
}

impl NewInnerNode {
    pub fn buffer_fits_type(buffer: &mut &[u8]) -> bool {
        !buffer.is_empty() && buffer[0] == 0x00
    }

    fn serialize(&self) -> Result<Vec<u8>, String> {
        if self.keys.is_empty() {
            return Err("No keys in leaf".to_string());
        }
        let type_of_node: u8 = 0x00;
        let key_type: u8 = match self.keys.first().unwrap() {
            SData::INT(_) => 0x00,
            SData::STRING(_) => 0x01,
        };
        let number_of_keys = self.keys.len() as u32;
        let serialized_keys_size: u32 = self.keys.iter().map(|key| key.serialized_size()).sum();
        let serialized_pointers_size = (self.pointers.len() * 8) as u32;
        let mut serialized_inner = Vec::with_capacity(
            (1 + 1 + 4 + serialized_keys_size + serialized_pointers_size)
                .try_into()
                .unwrap(),
        );
        serialized_inner.push(type_of_node);
        serialized_inner.push(key_type);
        serialized_inner.extend_from_slice(&number_of_keys.to_le_bytes());
        for key in &self.keys {
            let mut serialized_key = Vec::with_capacity(key.serialized_size() as usize);
            key.serialize(&mut serialized_key);
            serialized_inner.extend_from_slice(&serialized_key);
        }
        for pointer in &self.pointers {
            serialized_inner.extend_from_slice(&pointer.page.to_le_bytes());
            serialized_inner.extend_from_slice(&pointer.offset.to_le_bytes());
        }
        Ok(serialized_inner)
    }

    pub fn deserialize(buffer: &mut &[u8]) -> Result<NewInnerNode, String> {
        let type_of_node: u8 = buffer[0];
        if type_of_node != 0 {
            return Err("Given buffer dosent contain leaf node".to_string());
        }
        let key_type: u8 = buffer[0];
        let number_of_keys = {
            let number_of_keys_serilized: &[u8] = &buffer[1..3];
            LittleEndian::read_u16(&buffer)
        };
        *buffer = &buffer[1..];
        let keys = {
            let mut keys_deserilized = Vec::with_capacity(number_of_keys.try_into().unwrap());
            match key_type {
                0x00 => {
                    let value = i32::from_le_bytes(
                        buffer[0..4]
                            .try_into()
                            .map_err(|_| "Slice with incorrect length".to_string())?,
                    );
                    *buffer = &buffer[4..];
                    keys_deserilized.push(SData::INT(value));
                }
                0x01 => {
                    let len = u32::from_le_bytes(
                        buffer[0..4]
                            .try_into()
                            .map_err(|_| "Slice with incorrect length".to_string())?,
                    );
                    *buffer = &buffer[4..];
                    let str_value = String::from_utf8_lossy(&buffer[0..len as usize]).to_string();
                    *buffer = &buffer[len as usize..];
                    keys_deserilized.push(SData::STRING(str_value));
                }
                _ => Err("Unknown key type".to_string())?,
            }
            Ok::<Vec<SData>, String>(keys_deserilized)
        }?;
        let pointers = {
            let mut pointers_deserilized = Vec::with_capacity((number_of_keys + 1).into());
            for _ in 0..number_of_keys + 1 {
                let page = u32::from_le_bytes(
                    buffer[0..4]
                        .try_into()
                        .map_err(|_| "Slice with incorrect length".to_string())?,
                );
                *buffer = &buffer[4..];
                let offset = u16::from_le_bytes(
                    buffer[0..2]
                        .try_into()
                        .map_err(|_| "Slice with incorrect length".to_string())?,
                );
                *buffer = &buffer[4..];
                pointers_deserilized.push(TuplePointer { page, offset });
            }
            Ok::<Vec<TuplePointer>, String>(pointers_deserilized)
        }?;
        Ok(NewInnerNode { keys, pointers })
    }
}

impl NewInnerNode {
    pub fn get_node_pointer_for_key(&self, key: &SData) -> Result<TuplePointer, String> {
        match self.keys.binary_search(key) {
            Ok(index) => self
                .pointers
                .get(index)
                .cloned()
                .ok_or_else(|| "Pointer not found".to_string()),
            Err(index) => self
                .pointers
                .get(index)
                .cloned()
                .ok_or_else(|| "Pointer not found".to_string()),
        }
    }
}
