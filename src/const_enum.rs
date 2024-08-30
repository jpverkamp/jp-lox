#[macro_export]
macro_rules! const_enum {
    ($vis:vis $name:ident as $type:ty {
        $($value:ident => $char:expr),+
        $(,)?
    }) => {
        #[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $($value),+
        }

        impl $name {
            pub fn to_value(&self) -> $type {
                match *self {
                    $(
                        $name::$value => $char,
                    )+
                }
            }

            #[allow(dead_code)]
            pub fn values() -> Vec<$name> {
                vec![$($name::$value),+]
            }
        }

        impl TryFrom<$type> for $name {
            type Error = ();

            fn try_from(value: $type) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $char => Ok($name::$value),
                    )+
                    _ => Err(()),
                }
            }
        }
    };
}