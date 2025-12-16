macro_rules! wayland_protocol(
    ($path:expr, [$($imports:path),*]) => {
        pub use self::generated::client;

        mod generated {
            #![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
            #![allow(non_upper_case_globals,non_snake_case,unused_imports)]
            #![allow(missing_docs, clippy::all)]

            pub mod client {
                //! Client-side API of this protocol
                use wayland_client;
                use wayland_client::protocol::*;
                $(use $imports::{client::*};)*

                pub mod __interfaces {
                    use wayland_client::protocol::__interfaces::*;
                    $(use $imports::{client::__interfaces::*};)*
                    wayland_scanner::generate_interfaces!($path);
                }
                use self::__interfaces::*;

                wayland_scanner::generate_client_code!($path);
            }

        }
    }
);
