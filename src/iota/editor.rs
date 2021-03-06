extern crate rustbox;

use std::comm::{Receiver, Sender};
use std::num;
use std::io::{File, FileMode, FileAccess};

use super::Response;
use input::Input;
use cursor::Direction;
use keyboard::Key;
use view::View;


enum EventStatus {
    Handled(Response),
    NotHandled,
}


pub struct Editor<'e> {
    pub running: bool,
    pub sender: Sender<rustbox::Event>,

    events: Receiver<rustbox::Event>,
    view: View<'e>,
}

impl<'e> Editor<'e> {
    pub fn new(source: Input) -> Editor<'e> {
        let view = View::new(source);

        let (send, recv) = channel();
        Editor {
            sender: send,
            events: recv,
            view: view,
            running: false,
        }
    }

    pub fn handle_key_event(&mut self, key: u16, ch: u32) -> Response {
        let key_code = key as u32 + ch;
        let input_key: Option<Key> = num::from_u32(key_code);

        match self.handle_system_event(input_key) {
            EventStatus::Handled(response) => { response }
            EventStatus::NotHandled        => { Response::Continue }
        }
    }

    pub fn save_active_buffer(&mut self) {
        let lines = &self.view.buffer.lines;
        let path = Path::new(&self.view.buffer.file_path);

        let mut file = match File::open_mode(&path, FileMode::Open, FileAccess::Write) {
            Ok(f) => f,
            Err(e) => panic!("file error: {}", e),
        };

        for line in lines.iter() {
            let mut data = line.borrow().data.clone();
            data.push('\n' as u8);
            let result = file.write(data.as_slice());

            if result.is_err() {
                // TODO(greg): figure out what to do here.
                panic!("Something went wrong while writing the file");
            }
        }
    }

    pub fn draw(&mut self) {
        self.view.draw();
        self.view.draw_status();
        self.view.draw_cursor();
    }

    pub fn start(&mut self) {
        self.running = true;
        self.event_loop();
        self.main_loop();
    }

    fn main_loop(&mut self) {
        while self.running {
            self.view.clear();
            self.draw();
            rustbox::present();
            if let rustbox::Event::KeyEvent(_, key, ch) = self.events.recv() {
                if let Response::Quit = self.handle_key_event(key, ch) {
                    self.running = false;
                }
            }
        }
    }

    fn event_loop(&self) {
        // clone the sender so that we can use it in the proc
        let sender = self.sender.clone();

        spawn(proc() {
            loop {
                sender.send(rustbox::poll_event());
            }
        });
    }

    fn handle_system_event(&mut self, k: Option<Key>) -> EventStatus {
        use super::keyboard;

        let key = match k {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        match key {
            keyboard::UP        => { self.view.move_cursor(Direction::Up); }
            keyboard::DOWN      => { self.view.move_cursor(Direction::Down); }
            keyboard::LEFT      => { self.view.move_cursor(Direction::Left); }
            keyboard::RIGHT     => { self.view.move_cursor(Direction::Right); }
            keyboard::ENTER     => { self.view.insert_line(); }

            // Tab inserts 4 spaces, rather than a \t
            keyboard::TAB       => { self.view.insert_tab(); }

            keyboard::BACKSPACE => { self.view.delete_char(Direction::Left); }
            keyboard::DELETE    => { self.view.delete_char(Direction::Right); }
            keyboard::CTRL_S     => { self.save_active_buffer(); }
            keyboard::CTRL_Q     => { return EventStatus::Handled(Response::Quit) }
            keyboard::CTRL_R     => { self.view.resize(); }

            // TODO(greg): move these keys to event handlers of each mode
            // This block is for matching keys which will insert a char to the buffer
            _ => { self.view.insert_char(key.get_char().unwrap()) }
        }
        // event is handled and we want to keep the editor running
        EventStatus::Handled(Response::Continue)
    }

}

