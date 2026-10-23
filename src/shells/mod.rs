//! One file per shell: what that shell is, on any compositor.
//!
//! Everything in here is a fact about the shell itself, so it is written once
//! and shared by every compositor that supports it. What a shell looks like *on a
//! particular compositor* — where its config lives, how it starts — is not here;
//! that is the per-compositor [`Port`](crate::shell::Port) in
//! `compositor::<id>::shells::<shell>`.
//!
//! ```text
//! shells/dms.rs                     the package, the install plan, the logo
//! compositor/hyprland/shells/dms.rs  where its config is, how it starts there
//! compositor/<id>/shells/dms.rs      the same, for another compositor
//! ```
//!
//! There is deliberately no list of shells here. Which shells a compositor offers
//! is that compositor's decision and lives in its own registry, so a shell nobody
//! has registered costs nothing and a compositor never has to import a shell it
//! does not support.
//!
//! Both halves are declared `const` rather than `static`: `Shell::join` takes them
//! by value, and a `const` is inlined at every use rather than being one shared
//! address that a per-compositor `static` would have to borrow from.

pub mod ambxst;
pub mod caelestia;
pub mod dms;
pub mod end4pc;
pub mod impasto;
pub mod inir;
pub mod nandoroid;
pub mod noctalia;
pub mod sekaipop;

/// Every shell this program knows about, whatever compositor it belongs to.
///
/// For tests, and for anything that has to visit a shell without caring which
/// compositor it belongs to. The switch itself always goes through a compositor's
/// own registry instead.
///
/// By value rather than by reference: `IDENTITY` is a `const`, so each mention of it
/// is a fresh value, and there is no shared address to hand out.
#[cfg(test)]
pub fn every() -> Vec<crate::shell::Identity> {
    vec![
        noctalia::IDENTITY,
        dms::IDENTITY,
        caelestia::IDENTITY,
        ambxst::IDENTITY,
        inir::IDENTITY,
        end4pc::IDENTITY,
        nandoroid::IDENTITY,
        sekaipop::IDENTITY,
        impasto::IDENTITY,
    ]
}
