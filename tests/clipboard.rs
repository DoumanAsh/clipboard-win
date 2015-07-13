extern crate clipboard_win;

use clipboard_win::*;

#[test]
fn get_clipboard_seq_num_test() {
    assert!(get_clipboard_seq_num().is_some());
}

#[test]
fn set_clipboard_test() {
    let test_array = vec!["ololo", "1234", "1234567891234567891234567891234567891", "12345678912345678912345678912345678912"];
    for expected_string in test_array {
        assert!(set_clipboard(expected_string).is_ok());

        let result = get_clipboard();
        assert!(result.is_ok());
        let result = result.unwrap();

        println!("Clipboard: {}", result);
        println!("Expected: {}", expected_string);
        assert!(result == expected_string);
    }
}

#[test]
fn get_clipboard_test() {
    let result = get_clipboard();
    assert!(result.is_ok());

    println!("Clipboard: {}", result.unwrap());
}

#[test]
fn strlen_test() {
    let test_vec = vec![1, 2, 0, 3, 4];
    unsafe {
        assert!(rust_strlen(test_vec.as_ptr()) == 2);
        assert!(rust_strlen(test_vec.as_ptr().offset(2)) == 0);
    }
}
