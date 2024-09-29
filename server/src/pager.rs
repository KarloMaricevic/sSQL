mod Pager;

struct Pager {
    file_path: String,
    file_lenght: usize,
}

impl Pager {
    fn new(file_path: String) -> Self {
        Pager { file_path }
    }
}

struct Page {
    header: HeaderPage,
}

struct HeaderPage {
    page_size: i8,
    number_of_pages: i8,
    table_schema: Schema,
}

struct Schema {
    table: String,
    collumnns: Vec<String>,
}
