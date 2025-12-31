pub mod cli;
pub mod enums;
pub mod helpers;

#[macro_export]
macro_rules! impl_Display {
  ( $T:ty ) => {
    impl std::fmt::Display for $T {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self
          .to_possible_value()
          .expect("no values are skipped")
          .get_name()
          .fmt(f)
      }
    }
  };
}

#[macro_export]
macro_rules! svec {
  () => {vec![]};
  ( $elem:expr; $n:expr ) => {vec![$elem.to_string(); $n]};
  ( $($x:expr),+ $(,)? ) => {vec![$($x.to_string()),*]};
}

#[macro_export]
macro_rules! get_name {
  ( follow $T:expr, $client:ident, $lang:expr ) => {{
    let mut result = $T.name.clone();
    if let Ok(resource) = $T.follow(&$client).await {
      for name in resource.names.iter() {
        if let Ok(item) = name.language.follow(&$client).await
          && item.name == $lang
        {
          result = name.name.clone();
        }
      }
    }
    result
  }};
  ( $T:expr, $client:ident, $lang:expr ) => {{
    let mut result = $T.name.clone();
    for name in $T.names.iter() {
      if let Ok(item) = name.language.follow(&$client).await
        && item.name == $lang
      {
        result = name.name.clone();
      }
    }
    result
  }};
}
