#[macro_export]
macro_rules! char_enum {
    ($vis:vis $name:ident {
        $($value:ident => $char:expr),+
        $(,)?
    }) => {
        #[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $($value),+
        }

        impl $name {
            pub fn to_char(&self) -> char {
                match *self {
                    $(
                        $name::$value => $char,
                    )+
                }
            }
        }

        impl TryFrom<char> for $name {
            type Error = ();

            fn try_from(value: char) -> Result<Self, Self::Error> {
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