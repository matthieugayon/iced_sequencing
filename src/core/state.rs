#[macro_use]
extern crate machine;

machine!(
  enum Actions {
    Green { count: u8 },
    Orange,
    Red
  }
);
