// SPDX-License-Identifier: MPL-2.0

//! Bun 1.4.4 smoke suite for Asterinas NixOS (r2-asterinas qualification).

use nixos_test_framework::*;

nixos_test_main!();

#[nixos_test]
fn bun_version(nixos_shell: &mut Session) -> Result<(), Error> {
    nixos_shell.run_cmd_and_expect("bun --version", "1.4.4")?;
    Ok(())
}

#[nixos_test]
fn bun_eval_one_plus_one(nixos_shell: &mut Session) -> Result<(), Error> {
    nixos_shell.run_cmd_and_expect("bun -e 'console.log(1+1)'", "2")?;
    Ok(())
}
