/// Jumpcut integration tests
/// 
/// Tests must be run sequentially! (They rely on the contents of the alias file directory.)
/// cargo test -- --test-threads=1

extern crate assert_cmd;

use assert_cmd::prelude::*;
mod utils;
use utils::*;

#[test]
fn add_alias() {
    run_test(|| {
        let mut cmd = jc_cmd();
        let alias = "up";
        cmd.args(&["add", alias, "cd .."]).unwrap();
        assert!(alias_exists(alias));
    });
}

#[test]
fn remove_alias() {
    run_test(|| {
        let alias = "up";
        jc_cmd().args(&["add", alias, "cd .."]).unwrap();
        assert!(alias_exists(alias));

        jc_cmd().args(&["rm", alias]).unwrap();
        assert!(!alias_exists(alias));
    });
}

#[test]
fn exec_alias() {
    run_test(|| {
        let alias = "up";
        jc_cmd().args(&["add", alias, "cd .."]).unwrap();
        assert!(alias_exists(alias));

        let out = jc_cmd().args(&[alias]).output();
        assert_eq!(out_to_str(out), "cd ..\n");
    });
}

#[test]
fn exec_alias_params() {
    run_test(|| {
        let alias = "rename";
        jc_cmd().args(&["add", alias, "mv ?1 ?2"]).unwrap();
        assert!(alias_exists(alias));

        let out = jc_cmd().args(&[alias, "---", "foo", "bar"]).output();
        assert_eq!(out_to_str(out), "mv foo bar\n");
    });
}