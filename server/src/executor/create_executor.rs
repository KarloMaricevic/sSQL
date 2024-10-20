use std::{ops::Deref, rc::Rc};
use crate::buff::buff_pool::BuffPool;

// should i use data directly from page?

use crate::{
    bptree::page::{NewInnerNode, NewLeafNode},
    constants::PG_CLASS_NAME_INDEX_FILE,
    information_schema::SData,
    page::Page,
    parser::ast::SqlStatement,
};

fn create(statemant: crate::SqlStatement, pool: &mut BuffPool) -> Result<(), String> {
    if let SqlStatement::CreateTable {
        table_name,
        primary_key,
        columns,
    } = statemant
    {
        let descriptor = pool.get_descriptor(PG_CLASS_NAME_INDEX_FILE, 1)?;
        descriptor.lock_shared();
        let mut node = descriptor.get_buff().get_first_tuple()?;
        loop {
            if NewInnerNode::buffer_fits_type(node) {
                let inner_node = NewInnerNode::deserialize(node)?;
                let next_node_pointer =
                    inner_node.get_node_pointer_for_key(&SData::STRING(table_name))?;
                let descriptor_with_pointed_page =
                    pool.get_descriptor(PG_CLASS_NAME_INDEX_FILE, next_node_pointer.page)?;
                descriptor.unlock_shared();
                descriptor_with_pointed_page.lock_shared();
                descriptor = descriptor_with_pointed_page;
                node = descriptor_with_pointed_page.get_buff().get_tuple(next_node_pointer.offset)?;
            } else {
                let leaf_node = NewLeafNode::deserialize(node)?;
                if let Ok(_) = leaf_node
                    .keys
                    .binary_search(&SData::STRING(table_name.clone()))
                {
                    descriptor.unlock_shared();
                    return Err("pg_class with the given table name already exists".to_string());
                }
                break;
            }
        }
        Ok(())
    } else {
        Err("Not given select statment to execute".to_string())
    }
}
// provjeri da li nesto s tim imenom vec postoji
// load first page of pg_class.name, and go to leaf which would contain that name,
