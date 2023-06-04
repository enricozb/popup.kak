pub fn bash<S: AsRef<str>>(string: S) -> String {
  format!("$'{}'", string.as_ref().replace('\\', "\\\\").replace('\'', "\\'"))
}

pub fn kak<S: AsRef<str>>(string: S) -> String {
  format!("%§{}§", string.as_ref().replace('§', "§§"))
}
