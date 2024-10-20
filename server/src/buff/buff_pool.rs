use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Seek, SeekFrom},
    thread::sleep,
    time::Duration,
};

use crate::constants::{DEFAULT_BUFF_POOL_SIZE, PAGE_SIZE_IN_BYTES};

use super::{
    buff_desc::{BufferDesc, LockMode},
    spin_lock::SpinLock,
};

pub struct BuffPool {
    s_lock: SpinLock, // think about this
    max_frames: u32,
    descriptors: VecDeque<Box<BufferDesc>>,
}

impl BuffPool {
    pub fn create(capacity: Option<u32>) -> Result<Self, String> {
        let max_number_of_pages = capacity.unwrap_or(DEFAULT_BUFF_POOL_SIZE) / PAGE_SIZE_IN_BYTES;
        Ok(Self {
            s_lock: SpinLock::new(),
            max_frames: max_number_of_pages,
            descriptors: VecDeque::with_capacity(max_number_of_pages as usize),
        })
    }

    pub fn get_descriptor(
        &mut self,
        from_file: &str,
        page_number: u32,
    ) -> Result<&Box<BufferDesc>, String> {
        self.s_lock.lock();
        if let Some(index) = self
            .descriptors
            .iter()
            .position(|desc| desc.file_name == from_file && desc.page == page_number)
        {
            self.s_lock.unlock();
            return Ok(&self.descriptors[index]);
        }

        if self.max_frames >= self.descriptors.len() as u32 {
            let page_content = load_page_to_memory(from_file, page_number)?;
            let buff_description = Box::from(BufferDesc::new(from_file, page_number, page_content));
            self.descriptors.push_front(buff_description);
            self.s_lock.unlock();
            return Ok(self.descriptors.front().unwrap());
        } else {
            loop {
                if let Some(index) = self.descriptors.iter().position(|desc| {
                    desc.is_pinned && matches!(desc.get_lock_type(), LockMode::UNLOCKED)
                }) {
                    let page_content = load_page_to_memory(from_file, page_number)?;
                    let buff_description =
                        Box::from(BufferDesc::new(from_file, page_number, page_content));
                    self.descriptors.insert(index, buff_description);
                    self.s_lock.unlock();
                    return Ok(self.descriptors.front().unwrap());
                }
                sleep(Duration::from_millis(200));
            } // stupidest solution
        }
    }
}

fn load_page_to_memory(
    file_name: &str,
    page_number: u32,
) -> Result<[u8; PAGE_SIZE_IN_BYTES as usize], String> {
    let mut file = File::options()
        .read(true)
        .open(file_name)
        .map_err(|e| format!("Error opening file '{}': {}", file_name, e.to_string()))?;
    file.seek(SeekFrom::Start((PAGE_SIZE_IN_BYTES * page_number).into()))
        .map_err(|e| {
            format!(
                "Error seeking page {} in file '{}': {}",
                page_number,
                file_name,
                e.to_string()
            )
        })?;
    let page_content = {
        let mut content: [u8; PAGE_SIZE_IN_BYTES as usize] = [0; PAGE_SIZE_IN_BYTES as usize];
        file.read_exact(&mut content).map_err(|e| {
            format!(
                "Error reading page {} from file '{}': {}",
                page_number,
                file_name,
                e.to_string()
            )
        })?;
        content
    };
    return Ok(page_content);
}
