use super::Oid;

struct PgClass {
    oid: Oid, // indexed, unique
    name: String,  // indexed, unique
    kind: Kind,
    namespace: Oid,
}

enum Kind {
    NormalTable,
    Index,
}

struct PgIndex {
    oid: Oid, 
    table: Oid,
    indkey: i32, // defines on what column number is index created
}

struct PgAttribute {
    owner_id: Oid,
    name: String,
    num: i32, // defines column number   
    // data_type: CollumnType, 
}
/*  
    when searching for index in collumn:
        1. search OID od the table
        2. search PgIndex with OID of the table to get file name where index tree is saved
        3. search PgAttribute where num == indkey and  owner_id == PgIndex.table to get data type
 */