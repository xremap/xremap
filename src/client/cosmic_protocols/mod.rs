// SPDX-License-Identifier: GPL-3.0-only

//! This crate provides bindings to the COSMIC wayland protocol extensions.
//!
//! These bindings are built on top of the crates wayland-client and wayland-server.
//!
//! Each protocol module contains a `client` and a `server` submodules, for each side of the
//! protocol. The creation of these modules (and the dependency on the associated crate) is
//! controlled by the two cargo features `client` and `server`.

#![warn(missing_docs)]
#![forbid(improper_ctypes, unsafe_op_in_unsafe_fn)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(rustfmt, rustfmt_skip)]

#[macro_use]
mod protocol_macro;


pub mod toplevel_info {
    //! Receive information about toplevel surfaces.

    #[allow(missing_docs)]
    pub mod v1 {
        wayland_protocol!(
            "./src/client/cosmic_protocols/cosmic-toplevel-info-unstable-v1.xml",
            [crate::client::cosmic_protocols::workspace::v1]
        );
    }
}

pub mod workspace {
    //! Receive information about and control workspaces.

    #[allow(missing_docs)]
    pub mod v1 {
        wayland_protocol!(
            "./src/client/cosmic_protocols/cosmic-workspace-unstable-v1.xml",
            []
        );
    }
}
