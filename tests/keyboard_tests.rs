use vk_bot_api::keyboard::{ButtonColor, Keyboard};

#[test]
fn test_keyboard_creation() {
    let kb = Keyboard::new().add_text_button("Button 1", None, Some(ButtonColor::Primary));

    let json = kb.to_json();
    assert_eq!(json["one_time"], false);
    assert_eq!(json["inline"], false);

    let buttons = json["buttons"].as_array().unwrap();
    assert_eq!(buttons.len(), 1);

    let row = buttons[0].as_array().unwrap();
    assert_eq!(row.len(), 1);

    let btn = &row[0];
    assert_eq!(btn["action"]["type"], "text");
    assert_eq!(btn["action"]["label"], "Button 1");
    assert_eq!(btn["color"], "primary");
}

#[test]
fn test_inline_keyboard() {
    let kb = Keyboard::new_inline();
    let json = kb.to_json();
    assert_eq!(json["inline"], true);
}

#[test]
fn test_one_time_keyboard() {
    let kb = Keyboard::new_one_time();
    let json = kb.to_json();
    assert_eq!(json["one_time"], true);
}
