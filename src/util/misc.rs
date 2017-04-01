use target_info::Target;

include!(concat!(env!("OUT_DIR"), "/version.rs"));
include!(concat!(env!("OUT_DIR"), "/rustc_version.rs"));

#[cfg(feature = "final")]
const THIS_TRACK: &'static str = "nightly";
// ^^^ should be reset to "stable" or "beta" according to the release branch.

#[cfg(not(feature = "final"))]
const THIS_TRACK: &'static str = "unstable";
// ^^^ This gets used when we're not building a final release; should stay as "unstable".

/// Get the platform identifier.
pub fn platform() -> String {
    let env = Target::env();
    let env_dash = if env.is_empty() { "" } else { "-" };
    format!("{}-{}{}{}", Target::arch(), Target::os(), env_dash, env)
}

/// Get the standard version string for this software.
pub fn version() -> String {
    let commit_date = commit_date().replace("-", "");
    let date_dash = if commit_date.is_empty() { "" } else { "-" };
    format!("Toyrpc/v{}-{}{}{}/{}/rustc{}",
            env!("CARGO_PKG_VERSION"),
            THIS_TRACK,
            date_dash,
            commit_date,
            platform(),
            rustc_version())
}
