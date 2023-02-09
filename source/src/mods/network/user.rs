use std::io::{BufRead, Write};
use std::sync::Mutex;
use std::{io::BufReader, net::TcpStream};
use std::{sync::Arc, thread};

use once_cell::sync::Lazy;
static COUNT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
pub static mut UNTRUSTED_USERS: Lazy<Vec<User>> = Lazy::new(|| Vec::new());
pub static mut TRUSTED_USERS: Lazy<Vec<User>> = Lazy::new(|| Vec::new());
pub struct User {
    pub isok: bool,
    pub id: u16,
    pub user: Arc<TcpStream>,
}
impl User {
    pub fn read_thread(&self) {
        let stream2 = Arc::clone(&self.user);
        let id = self.id;
        thread::spawn(move || {
            let mut reader = BufReader::new(&*stream2);
            println!("");
            loop {
                let mut line = String::new();
                let bytes = reader.read_line(&mut line).unwrap();
                if bytes == 0 {
                    println!("接続終了");
                    let mut remove = 0;
                    let _aru = false;
                    unsafe {
                        for idx in 0..UNTRUSTED_USERS.len() {
                            if UNTRUSTED_USERS[idx].id == id {
                                remove = idx;
                            }
                        }
                        UNTRUSTED_USERS.remove(remove);
                    }

                    break;
                }
                println!("{}", line);
            }
        });
    }
    pub fn write(&self, context: String) {
        (&*self.user).write_all(context.as_bytes()).unwrap();
        (&*self.user).flush().unwrap();
    }
}
pub fn init(stream: Arc<TcpStream>) -> User {
    *COUNT.lock().unwrap() += 1;
    User {
        user: stream,
        id: *COUNT.lock().unwrap(),
        isok: true,
    }
}
