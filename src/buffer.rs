use std::io::{File, BufferedReader};
use std::cell::RefCell;

use utils;
use cursor::{Direction, Cursor};


pub struct Buffer {
    pub file_path: String,
    pub lines: Vec<RefCell<Line>>,

    pub cursor: Cursor,
}

impl Buffer {
    /// Create a new buffer instance
    pub fn new() -> Buffer {
        Buffer {
            file_path: String::new(),
            lines: Vec::new(),
            cursor: Cursor::new(),
        }
    }

    /// Create a new buffer instance and load the given file
    pub fn new_from_file(path: &Path) -> Buffer {
        let mut file = BufferedReader::new(File::open(path));
        let lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();
        let mut buffer = Buffer::new();

        buffer.file_path = path.as_str().unwrap().to_string();

        // for every line in the file we add a corresponding line to the buffer
        for line in lines.iter() {
            let mut data = line.clone();
            // remove \n chars
            data.pop();
            buffer.lines.push(RefCell::new(Line::new(data)));
        }

        buffer
    }

    /// Draw the contents of the buffer
    ///
    /// Loops over each line in the buffer and draws it to the screen
    pub fn draw_contents(&self) {
        for (index, line) in self.lines.iter().enumerate() {
            let ln = line.borrow();
            utils::draw(index, ln.data.clone());
        }
    }

    pub fn draw_status(&self) {
        let height = utils::get_term_height();
        let (cursor_x, cursor_y) = self.cursor.get_position();
        let data = self.file_path.clone();
        let line_count = self.lines.len();
        utils::draw(
            height - 1,
            format!("{}, cursor: {}-{}, termwidth: {}, termheight: {}, lines: {}",
                    data, cursor_x, cursor_y, utils::get_term_height(), utils::get_term_width(), line_count));
    }

    pub fn adjust_cursor(&mut self, dir: Direction) {
        let (mut x, mut y) = self.cursor.get_position();
        match dir {
            Direction::Up => {
                let line = self.get_line_at(y-1);
                if line.is_some() {
                    y -= 1;
                    // if the current cursor offset is after the end of the
                    // previous line, move the offset back to the end of the line
                    let line_len = line.unwrap().borrow().data.len();
                    if x > line_len {
                        x = line_len;
                    }
                }
            }
            Direction::Down => {
                let line = self.get_line_at(y+1);
                if line.is_some() {
                    y += 1;
                    // if the current cursor offset is after the end of the
                    // next line, move the offset back to the end of the line
                    let line_len = line.unwrap().borrow().data.len();
                    if x > line_len {
                        x = line_len;
                    }
                }
            }
            Direction::Right => {
                let line = &self.get_line_at(y);
                if line.is_some() && line.unwrap().borrow().len() > x {
                    x += 1
                }
            }
            Direction::Left => {
                let line = &self.get_line_at(y);
                if line.is_some() && x > 0 {
                    x -= 1
                }
            }
        }
        self.cursor.set_position(x, y);
    }

    pub fn delete_char(&mut self) {
        let (offset, line_num) = self.cursor.get_position();

        if offset == 0 {
            return self.join_line_with_previous(line_num);
        }

        let bits = self.split_line();
        let mut data = bits[0].clone();
        data.pop();

        let new_data = format!("{}{}", data, bits[1]);

        {
            let line = self.get_line_at(line_num);
            line.unwrap().borrow_mut().data = new_data;
        }
        self.cursor.set_position(offset - 1, line_num);
    }

    pub fn insert_char(&mut self, ch: char) {
       let (mut x, y) = self.cursor.get_position();
       {
           let line = &self.get_line_at(y);

           // get Vec<u8> from the current line contents
           let mut data = line.unwrap().borrow().data.clone().into_bytes();

           // add the new character to the Vec at the cursors `x` position
           data.insert(x, ch as u8);

           // convert to Vec back into a string
           let new_data = String::from_utf8(data);

           if new_data.is_ok() {
               // update the line
               line.unwrap().borrow_mut().data = new_data.unwrap();
           }
           x += 1;
       }
       self.cursor.set_position(x, y);

    }

    pub fn insert_new_line(&mut self) {
        let line_num = self.cursor.get_linenum();

        // split the current line at the cursor position
        let bits = &self.split_line();
        self.update_line(bits.clone());

        // move the cursor down and to the start of the next line
        self.cursor.set_position(0, line_num + 1);
    }

    /// Join the line identified by `line_num` with the one at `line_num - 1 `.
    fn join_line_with_previous(&mut self, line_num: uint) {
        let mut current_line_data: String;
        let mut prev_line_data: String;
        let line_len: uint;
        {
            let prev_line = self.get_line_at(line_num - 1);
            if prev_line.is_none() {
                return
            }
            prev_line_data = prev_line.unwrap().borrow().data.clone();
            line_len = prev_line_data.len();
        }
        {
            // get current line data
            let current_line = self.get_line_at(line_num);
            current_line_data = current_line.unwrap().borrow().data.clone();
        }
        {
            // append current line data to prev line
            // FIXME: this is duplicated above in a different scope...
            let prev_line = self.get_line_at(line_num - 1).unwrap();

            let new_data = format!("{}{}", prev_line_data, current_line_data);
            prev_line.borrow_mut().data = new_data;
        }

        // clear the line
        utils::clear_line(line_num);
        // remove current line
        self.lines.remove(line_num);
        // move the cursor
        self.cursor.set_position(line_len, line_num - 1);
    }

    fn update_line(&mut self, mut bits: Vec<String>) {
        let line_num = self.cursor.get_linenum();
        {
            // truncate the current line
            let line = &self.get_line_at(line_num);
            line.unwrap().borrow_mut().data = bits.remove(0).unwrap();
        }

        // add new line below current
        utils::clear_line(line_num+1);
        self.lines.insert(line_num+1, RefCell::new(Line::new(bits.remove(0).unwrap())));
    }

    fn split_line(&mut self) -> Vec<String> {
        let (x, y) = self.cursor.get_position();
        let line = &self.get_line_at(y);

        let data = line.unwrap().borrow().data.clone().into_bytes();
        let old_data = data.slice_to(x);
        let new_data = data.slice_from(x);

        vec!(
            String::from_utf8_lossy(old_data).into_string(),
            String::from_utf8_lossy(new_data).into_string(),
        )
    }

    fn get_line_at(&mut self, line_num: uint) -> Option<&RefCell<Line>> {
        for (index, line) in self.lines.iter().enumerate() {
            if index == line_num {
                return Some(line)
            }
        }
        None
    }

}


pub struct Line {
    pub data: String,
}

impl Line {
    /// Create a new line instance
    pub fn new(data: String) -> Line {
        Line{
            data: data,
        }
    }

    /// Get the length of the current line
    pub fn len(&self) -> uint {
        self.data.len()
    }
}


