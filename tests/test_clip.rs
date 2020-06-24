use clipboard_win::{Getter, Setter, Clipboard};
use clipboard_win::formats::{RawData, Unicode, CF_TEXT};

macro_rules! os_assert {
    ($expr:expr) => {
        if (!$expr) {
            panic!("Assertion {} failed: {}", stringify!($expr), std::io::Error::last_os_error());
        }
    }
}

fn should_work_with_string() {
    let text = "For my waifu\n!";

    let _clip = Clipboard::new_attempts(10).unwrap();

    os_assert!(Unicode.write_clipboard(&text));

    let mut output = String::new();

    os_assert!(Unicode.read_clipboard(&mut output) == text.len());

    assert_eq!(text, output);

    os_assert!(Unicode.read_clipboard(&mut output) == text.len());

    assert_eq!(format!("{0}{0}", text), output);
}

fn should_work_with_wide_string() {
    let text = "メヒーシャ!";

    let _clip = Clipboard::new_attempts(10).unwrap();

    os_assert!(Unicode.write_clipboard(&text));

    let mut output = String::new();

    os_assert!(Unicode.read_clipboard(&mut output) == text.len());

    assert_eq!(text, output);

    os_assert!(Unicode.read_clipboard(&mut output) == text.len());

    assert_eq!(format!("{0}{0}", text), output);
}

fn should_work_with_bytes() {
    let text = "Again waifu!?\0";

    let ascii = RawData(CF_TEXT);
    let _clip = Clipboard::new_attempts(10).unwrap();

    os_assert!(ascii.write_clipboard(&text));

    let mut output = String::with_capacity(text.len() * 2);

    {
        let output = unsafe { output.as_mut_vec() };

        os_assert!(ascii.read_clipboard(output) == text.len());
    }

    assert_eq!(text, output);

    {
        let output = unsafe { output.as_mut_vec() };

        os_assert!(ascii.read_clipboard(output) == text.len());
    }

    assert_eq!(format!("{0}{0}", text), output);
}

macro_rules! run {
    ($name:ident) => {
        println!("Clipboard test: {}...", stringify!($name));
        $name();
    }
}

#[test]
fn clipboard_should_work() {
    run!(should_work_with_string);
    run!(should_work_with_wide_string);
    run!(should_work_with_bytes);
}
