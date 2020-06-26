use clipboard_win::{Getter, Setter, Clipboard};
use clipboard_win::formats::{RawData, Unicode, Bitmap, CF_TEXT};

fn should_work_with_bitmap() {
    let _clip = Clipboard::new_attempts(10).expect("Open clipboard");

    let test_image_bytes = std::fs::read("tests/test-image.bmp").expect("Read test image");
    Bitmap.write_clipboard(&test_image_bytes).expect("To set image");

    let mut out = Vec::new();

    assert_eq!(Bitmap.read_clipboard(&mut out).expect("To get image"), out.len());

    assert_eq!(test_image_bytes.len(), out.len());
    assert!(test_image_bytes == out);
}

fn should_work_with_string() {
    let text = "For my waifu\n!";

    let _clip = Clipboard::new_attempts(10).expect("Open clipboard");

    Unicode.write_clipboard(&text).expect("Write text");

    let mut output = String::new();

    assert_eq!(Unicode.read_clipboard(&mut output).expect("Read text"), text.len());
    assert_eq!(text, output);

    assert_eq!(Unicode.read_clipboard(&mut output).expect("Read text"), text.len());
    assert_eq!(format!("{0}{0}", text), output);
}

fn should_work_with_wide_string() {
    let text = "メヒーシャ!";

    let _clip = Clipboard::new_attempts(10).expect("Open clipboard");

    Unicode.write_clipboard(&text).expect("Write text");

    let mut output = String::new();

    assert_eq!(Unicode.read_clipboard(&mut output).expect("Read text"), text.len());
    assert_eq!(text, output);

    assert_eq!(Unicode.read_clipboard(&mut output).expect("Read text"), text.len());
    assert_eq!(format!("{0}{0}", text), output);
}

fn should_work_with_bytes() {
    let text = "Again waifu!?\0";

    let ascii = RawData(CF_TEXT);
    let _clip = Clipboard::new_attempts(10).expect("Open clipboard");

    ascii.write_clipboard(&text).expect("Write ascii");

    let mut output = String::with_capacity(text.len() * 2);

    {
        let output = unsafe { output.as_mut_vec() };
        assert_eq!(ascii.read_clipboard(output).expect("read ascii"), text.len());
    }

    assert_eq!(text, output);

    {
        let output = unsafe { output.as_mut_vec() };
        assert_eq!(ascii.read_clipboard(output).expect("read ascii"), text.len());
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
    run!(should_work_with_bitmap);
    run!(should_work_with_string);
    run!(should_work_with_wide_string);
    run!(should_work_with_bytes);
}