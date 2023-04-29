use serde::{Deserialize, Deserializer};
use serde_tuple::Serialize_tuple;
use serde_repr::{Serialize_repr, Deserialize_repr};
use num_enum::TryFromPrimitive;
use serde_json::Value;

#[derive(Serialize_tuple, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct RevlogEntry {
  pub id: u64,
  pub cid: u64,
  pub usn: u64,
  /// - In the V1 scheduler, 3 represents easy in the learning case.
  /// - 0 represents manual rescheduling.
  #[serde(rename = "ease")]
  pub button_chosen: u8,
  /// Positive values are in days, negative values in seconds.
  #[serde(rename = "ivl", deserialize_with = "deserialize_int_from_number")]
  pub interval: i32,
  /// Positive values are in days, negative values in seconds.
  #[serde(rename = "lastIvl", deserialize_with = "deserialize_int_from_number")]
  pub last_interval: i32,
  /// Card's ease after answering, stored as 10x the %, eg 2500 represents
  /// 250%.
  #[serde(rename = "factor", deserialize_with = "deserialize_int_from_number")]
  pub ease_factor: u32,
  /// Amount of milliseconds taken to answer the card.
  #[serde(rename = "time", deserialize_with = "deserialize_int_from_number")]
  pub taken_millis: u32,
  #[serde(rename = "type", default, deserialize_with = "default_on_invalid")]
  pub review_kind: RevlogReviewKind,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum RevlogReviewKind {
    Learning = 0,
    Review = 1,
    Relearning = 2,
    /// Old Anki versions called this "Cram" or "Early", and assigned it when
    /// reviewing cards ahead. It is now only used for filtered decks with
    /// rescheduling disabled.
    Filtered = 3,
    Manual = 4,
}

impl Default for RevlogReviewKind {
    fn default() -> Self {
        RevlogReviewKind::Learning
    }
}

/// Note: if you wish to cover the case where a field is missing, make sure you
/// also use the `serde(default)` flag.
fn default_on_invalid<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    Ok(T::deserialize(v).unwrap_or_default())
}

fn deserialize_int_from_number<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: serde::Deserialize<'de> + FromI64,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrFloat {
        Int(i64),
        Float(f64),
    }

    match IntOrFloat::deserialize(deserializer)? {
        IntOrFloat::Float(f) => Ok(T::from_i64(f as i64)),
        IntOrFloat::Int(i) => Ok(T::from_i64(i)),
    }
}

trait FromI64 {
  fn from_i64(val: i64) -> Self;
}

impl FromI64 for i32 {
  fn from_i64(val: i64) -> Self {
      val as Self
  }
}

impl FromI64 for u32 {
  fn from_i64(val: i64) -> Self {
      val.max(0) as Self
  }
}

impl FromI64 for i64 {
  fn from_i64(val: i64) -> Self {
      val
  }
}

fn main() {
  let a = RevlogEntry{
    id: 123,
    cid: 456,
    usn: 789,
    button_chosen: 3,
    interval: 3,
    last_interval: 3,
    ease_factor: 2,
    taken_millis: 10,
    review_kind: RevlogReviewKind::Manual,
  };

  let s = serde_json::to_string(&a);
  println!("{s:#?}");
}