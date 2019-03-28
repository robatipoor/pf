#[cfg(test)]
mod integration {
    use assert_cmd::prelude::*;
    use std::process::*;
    #[test]
    fn pip_string_to_pf_cmd_test() {
        let mut echo = Command::new("echo")
            .arg("Hello")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let echo_out = echo.stdout.take().unwrap();
        echo.wait().unwrap();
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let pf = cmd.stdin(echo_out).stdout(Stdio::piped()).spawn().unwrap();
        let out = pf.wait_with_output().unwrap();
        let out_str = std::str::from_utf8(&out.stdout).unwrap();
        cmd.args(&["-f",out_str]).assert().success();
        cmd.args(&["-d",out_str]).assert().success();
    }
}
