//// A module comment containing ordinary prose.

/// A normal documentation comment with an email@example.com mention.
/// @codegen json
/// @json tag_style "adjacent"
/// @firestore collection "users"
/// @unknown future_value
pub type User {
  User(id: String)
}

pub fn ordinary_code() {
  "@codegen"
}
