extern crate clipboard_win;

use clipboard_win::*;
use clipboard_win::wrapper::{open_clipboard, close_clipboard, set_clipboard_raw, register_format, count_formats, is_format_avail, get_clipboard_seq_num};

//NOTE: parallel running may cause fail.

#[test]
fn win_error_test() {
    let result = WindowsError::new(0);
    println!("WinError({})={}", &result.errno(), result.errno_desc());

    assert!(result == WindowsError::new(0));
    assert!(WindowsError::new(1) != result);

    let result = WindowsError::new(666);
    println!("WinError({})={}", &result.errno(), result.errno_desc());
}

#[test]
fn get_clipboard_formats_test() {
    let clipboard_formats = get_clipboard_formats();

    assert!(clipboard_formats.is_ok());

    let clipboard_formats = clipboard_formats.unwrap();
    println!("get_clipboard_formats_test: clipboard formats: {:?}", clipboard_formats);
    for format in clipboard_formats {
        if let Some(format_name) = get_format_name(format) {
            println!("{}={}", format, format_name);
        }
    }
}

#[test]
fn get_clipboard_seq_num_test() {
    assert!(get_clipboard_seq_num().is_some());
}

#[test]
fn set_clipboard_test() {
    let test_array = vec!["ololo", "1234", "1234567891234567891234567891234567891", "12345678912345678912345678912345678912"];
    for expected_string in test_array {
        assert!(set_clipboard(expected_string).is_ok());

        let result = get_clipboard_string();
        assert!(result.is_ok());
        let result = result.unwrap();

        println!("set_clipboard_test: Clipboard: {}", result);
        println!("set_clipboard_test: Expected: {}", expected_string);
        assert!(result == expected_string);
    }
}

#[test]
fn get_clipboard_test() {
    let result = get_clipboard_string();
    assert!(result.is_ok());

    println!("get_clipboard_test: Clipboard: {}", result.unwrap());
}

#[test]
fn is_format_avail_test() {
    assert!(is_format_avail(13)); //default unicode format
    assert!(!is_format_avail(66613666));
}

#[test]
fn count_formats_test() {
    let result = count_formats();

    assert!(result.is_ok());

    //Now it is a bit bad test, but generally there should be:
    // 4 - text
    // 11 - link
    // 13 - file/directory
    let result = result.unwrap();
    println!("count_formats_test: number of formats={}", result);
    assert!(result == 4 || result == 11 || result == 13);
}

#[test]
fn register_format_test() {
    let new_format = register_format("text");
    assert!(new_format.is_ok());

    let new_format = new_format.unwrap();
    println!("register_format_test: new_format={}", new_format);
    assert!(open_clipboard().is_ok());

    let expect_buf = [13, 12, 122, 1];
    println!("register_format_test: set clipboard={:?}", expect_buf);
    assert!(set_clipboard_raw(&expect_buf, new_format).is_ok());
    assert!(close_clipboard().is_ok());
    assert!(is_format_avail(new_format));

    let result = get_clipboard(new_format);
    assert!(result.is_ok());
    let result = result.unwrap();

    assert!(result == expect_buf);
    println!("register_format_test: saved clipboard={:?}", result);
}
