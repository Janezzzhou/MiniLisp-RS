use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static OUTPUT_BUFFER: RefCell<Option<Rc<RefCell<String>>>> = const { RefCell::new(None) };
}

pub fn write(text: &str) {
    OUTPUT_BUFFER.with(|buffer| {
        if let Some(buffer) = buffer.borrow().as_ref() {
            buffer.borrow_mut().push_str(text);
        } else {
            print!("{}", text);
        }
    });
}

pub fn writeln(text: &str) {
    write(text);
    write("\n");
}

pub fn newline() {
    write("\n");
}

pub fn capture<F, T>(f: F) -> (String, T)
where
    F: FnOnce() -> T,
{
    let buffer = Rc::new(RefCell::new(String::new()));

    OUTPUT_BUFFER.with(|slot| {
        let previous = slot.replace(Some(buffer.clone()));
        let result = f();
        let output = buffer.borrow().clone();
        slot.replace(previous);
        (output, result)
    })
}
