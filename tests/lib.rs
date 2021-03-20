extern crate botzilla;

#[test]
fn called_from_outside() {
    assert_eq!(botzilla::hello_world(0), Some("Hello Sol!"));
}
