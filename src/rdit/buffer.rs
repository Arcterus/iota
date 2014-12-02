use std::io::fs::PathExtensions;
use std::io::{File, BufferedReader};
use std::cell::RefCell;

use utils;


pub struct Buffer {
    pub file_path: String,
    pub lines: Vec<RefCell<Line>>,
}

impl Buffer {
    /// Create a new buffer instance
    pub fn new() -> Buffer {
        Buffer {
            file_path: String::new(),
            lines: Vec::new(),
        }
    }

    /// Create a new buffer with a single line
    pub fn new_empty() -> Buffer {
        let mut buffer = Buffer::new();
        buffer.lines.push(RefCell::new(Line::new(String::new(), 0)));
        buffer.file_path = String::from_str("untitled");

        buffer
    }

    /// Create a new buffer instance and load the given file
    pub fn new_from_file(path: &Path) -> Buffer {
        let mut buffer = Buffer::new();

        if path.exists() {
            let mut file = BufferedReader::new(File::open(path));
            let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();

            // for every line in the file we add a corresponding line to the buffer
            for (index, line) in lines.iter().enumerate() {
                let mut data = line.clone();
                // remove \n chars
                data.pop();
                buffer.lines.push(RefCell::new(Line::new(data, index)));
            }
        } else {
            buffer.lines.push(RefCell::new(Line::new(String::new(), 0)));
        }

        buffer.file_path = path.as_str().unwrap().to_string();
        buffer
    }

    pub fn get_status_text(&self) -> String {
        let file_path = self.file_path.clone();
        let line_count = self.lines.len();
        format!("{}, lines: {}", file_path, line_count)
    }

    fn fix_linenums(&mut self) {
        for (index, line) in self.lines.iter().enumerate() {
            line.borrow_mut().linenum = index;
        }
    }

    pub fn insert_line(&mut self, offset: uint, mut line_num: uint) {
        // split the current line at the cursor position
        let bits = &self.split_line(offset, line_num);
        {
            // truncate the current line
            let line = &self.get_line_at(line_num);
            line.unwrap().borrow_mut().data.truncate(offset);
        }

        line_num += 1;

        // add new line below current
        utils::clear_line(line_num);
        self.lines.insert(line_num, RefCell::new(Line::new(bits.clone().remove(1).unwrap(), line_num)));

        self.fix_linenums();
    }

    /// Join the line identified by `line_num` with the one at `line_num - 1 `.
    pub fn join_line_with_previous(&mut self, offset: uint, line_num: uint) -> uint {
        let mut current_line_data: String;
        let mut prev_line_data: String;
        let line_len: uint;
        {
            let prev_line = self.get_line_at(line_num - 1);
            if prev_line.is_none() {
                return offset
            }
            prev_line_data = prev_line.unwrap().borrow().data.clone();
            line_len = prev_line_data.len();
        }

        {
            // get current line data
            let current_line = self.get_line_at(line_num).unwrap();
            current_line_data = current_line.borrow().data.clone();
        }

        {
            // append current line data to prev line
            // FIXME: this is duplicated above in a different scope...
            let prev_line = self.get_line_at(line_num - 1).unwrap();

            let new_data = format!("{}{}", prev_line_data, current_line_data);
            prev_line.borrow_mut().data = new_data;
        }

        utils::clear_line(line_num);
        self.lines.remove(line_num);

        self.fix_linenums();

        // return cursor offset
        return line_len
    }

    /// Split the line identified by `line_num` at `offset`
    fn split_line(&mut self, offset: uint, line_num: uint) -> Vec<String> {
        let line = self.get_line_at(line_num).unwrap();

        let data = line.borrow().data.clone().into_bytes();
        let old_data = data.slice_to(offset);
        let new_data = data.slice_from(offset);

        vec!(
            String::from_utf8_lossy(old_data).into_string(),
            String::from_utf8_lossy(new_data).into_string(),
        )
    }

    fn get_line_at(&self, line_num: uint) -> Option<&RefCell<Line>> {
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        Some(&self.lines[line_num])
    }

}


pub struct Line {
    pub data: String,
    pub linenum: uint,
}

impl Line {
    /// Create a new line instance
    pub fn new(data: String, line_num: uint) -> Line {
        Line{
            data: data,
            linenum: line_num,
        }
    }

    /// Get the length of the current line
    pub fn len(&self) -> uint {
        self.data.len()
    }
}


#[cfg(test)]
mod tests {

    use std::cell::RefCell;
    use buffer::Buffer;
    use buffer::Line;

    fn setup_buffer() -> Buffer {
        let mut buffer = Buffer::new();
        buffer.file_path = String::from_str("/some/file.txt");
        buffer.lines = vec!(
            RefCell::new(Line::new("test".to_string(), 0)),
            RefCell::new(Line::new("".to_string(), 1)),
            RefCell::new(Line::new("text file".to_string(), 2)),
            RefCell::new(Line::new("content".to_string(), 3)),
        );
        buffer
    }

    #[test]
    fn test_get_status_text() {
        let buffer = setup_buffer();
        assert_eq!(buffer.get_status_text(), "/some/file.txt, lines: 4".to_string())
    }

    #[test]
    fn test_insert_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(0, 0);
        assert_eq!(buffer.lines.len(), 5);
    }

    #[test]
    fn test_insert_line_in_middle_of_other_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(1, 0);
        assert_eq!(buffer.lines.len(), 5);

        let ref line = buffer.lines[1];
        assert_eq!(line.borrow().data, "est".to_string());
    }

    #[test]
    fn test_line_numbers_are_fixed_after_adding_new_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(1, 2);
        assert_eq!(buffer.lines.len(), 5);
        
        // check that all linenums are sequential
        for (index, line) in buffer.lines.iter().enumerate() {
            assert_eq!(index, line.borrow().linenum);
        }
    }

    #[test]
    fn test_join_line_with_previous() {
        let mut buffer = setup_buffer();

        let offset = buffer.join_line_with_previous(0, 3);

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[2].borrow().data, "text filecontent".to_string());
        assert_eq!(offset, 9);
    }

    #[test]
    fn test_split_line() {
        let mut buffer = setup_buffer();
        let segments = buffer.split_line(3, 3);

        assert_eq!(segments, vec!("con".to_string(), "tent".to_string()));
    }
}
