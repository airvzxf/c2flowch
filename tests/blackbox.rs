use std::fs;
use tempfile::tempdir;

#[test]
fn test_unidad_02_actividad_ea() {
    test_file("unidad_02-actividad_ea");
}

#[test]
fn test_unidad_02_actividad_03() {
    test_file("unidad_02-actividad_03");
}

#[test]
fn test_unidad_03_actividad_02() {
    test_file("unidad_03-actividad_02");
}

#[test]
fn test_unidad_03_actividad_03() {
    test_file("unidad_03-actividad_03");
}

fn test_file(base_name: &str) {
    let dir = tempdir().expect("Could not create temp dir");
    let c_file = format!("resources/c/{}.c", base_name);
    let expected_yaml_file = format!("resources/c/{}.yaml", base_name);
    let expected_ascii_file = format!("resources/ascii/{}.txt", base_name);

    let output_yaml = dir.path().join(format!("{}.yaml", base_name));
    let output_ascii = dir.path().join(format!("{}.txt", base_name));

    // 1. Parse C to YAML
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("c2flowch");
    cmd.arg("parse")
        .arg(&c_file)
        .arg("--output")
        .arg(&output_yaml)
        .assert()
        .success();

    let actual_yaml = fs::read_to_string(&output_yaml).expect("Could not read output YAML");
    let expected_yaml =
        fs::read_to_string(&expected_yaml_file).expect("Could not read expected YAML");
    assert_eq!(
        actual_yaml, expected_yaml,
        "YAML output mismatch for {}",
        base_name
    );

    // 2. Render YAML to ASCII
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("c2flowch");
    cmd.arg("render")
        .arg(&output_yaml)
        .arg("--output")
        .arg(&output_ascii)
        .assert()
        .success();

    let actual_ascii = fs::read_to_string(&output_ascii).expect("Could not read output ASCII");
    let expected_ascii =
        fs::read_to_string(&expected_ascii_file).expect("Could not read expected ASCII");
    assert_eq!(
        actual_ascii, expected_ascii,
        "ASCII output mismatch for {}",
        base_name
    );
}
