extern crate clipboard_win;

use clipboard_win::*;

#[test]
fn set_clipboard_test() {
    let expected_string = "ololo";
    set_clipboard(expected_string);

    let result = get_clipboard();
    assert!(result.is_ok());
    let result = result.unwrap();

    println!("Clipboard: {}", result);
    println!("Expected: {}", expected_string);
    assert!(result == expected_string);

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
    let test_str = "12345\0".to_string();
    unsafe {
        assert!(rust_strlen(test_vec.as_ptr()) == 2);
        assert!(rust_strlen(test_vec.as_ptr().offset(2)) == 0);
        assert!(rust_strlen(test_str.as_ptr()) == test_str.len() - 1);
    }
}
