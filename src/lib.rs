//! A hello world crate with automated testing, documentation,
//! continuous integration, tested example code, implemented as a library
//! with a command line tool. Supports Sol and all its planets, not just 'Hello (unspecified) world!'

/// This function says hello to one of Sol's worlds, specified by a numeric argument
///
/// # Examples
///
/// ```
/// let greeting = botzilla::hello_world(3);
///
/// assert_eq!(greeting, Some("Hello Earth!"));
/// ```
pub fn hello_world(planet: u8) -> Option<&'static str> {
  return match planet {
    0 => Some("Hello Sol!"),
    1 => Some("Hello Mercury!"),
    2 => Some("Hello Venus!"),
    3 => Some("Hello Earth!"),
    4 => Some("Hello Mars!"),
    5 => Some("Hello Jupiter!"),
    6 => Some("Hello Saturn!"),
    7 => Some("Hello Uranus!"),
    8 => Some("Hello Neptune!"),
    _ => None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hello_sol() {
    assert_eq!(hello_world(0), Some("Hello Sol!"));
  }

  #[test]
  fn hello_mercury() {
    assert_eq!(hello_world(1), Some("Hello Mercury!"));
  }

  #[test]
  fn hello_venus() {
    assert_eq!(hello_world(2), Some("Hello Venus!"));
  }

  #[test]
  fn hello_earth() {
    assert_eq!(hello_world(3), Some("Hello Earth!"));
  }

  #[test]
  fn hello_mars() {
    assert_eq!(hello_world(4), Some("Hello Mars!"));
  }

  #[test]
  fn hello_jupiter() {
    assert_eq!(hello_world(5), Some("Hello Jupiter!"));
  }

  #[test]
  fn hello_saturn() {
    assert_eq!(hello_world(6), Some("Hello Saturn!"));
  }

  #[test]
  fn hello_uranus() {
    assert_eq!(hello_world(7), Some("Hello Uranus!"));
  }

  #[test]
  fn hello_neptune() {
    assert_eq!(hello_world(8), Some("Hello Neptune!"));
  }

  #[test]
  fn pluto_isnt_a_planet() {
    assert_eq!(hello_world(9), None);
  }
}
