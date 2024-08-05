pub trait StringHelpers {
    fn take_until(&self, until: &[char]) -> String;
    fn extract_integer(&self) -> Option<String>;
}


impl StringHelpers for str {

    fn take_until(&self, until: &[char]) -> String {
        for (i, c) in self.char_indices() {
            if until.contains(&c) {
                return self[..i].to_string();
            }
        }
        self.to_string()
    }


    fn extract_integer(&self) -> Option<String> {
        print!("Given string is {self}");
        for (i, c) in self.chars().enumerate() {
            if !c.is_numeric() {
                if i == 0 {
                    return None;
                } else {
                    return Some(self[..i].to_string());
                }
            }
        }
        Some(self.to_string())
    }
}
