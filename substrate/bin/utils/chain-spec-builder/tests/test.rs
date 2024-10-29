// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::fs::File;

use clap::Parser;
use cmd_lib::run_cmd;
use sc_chain_spec::update_code_in_json_chain_spec;
use staging_chain_spec_builder::ChainSpecBuilder;

// note: the runtime path will not be read, runtime code will be set directly, to avoid hassle with
// creating the wasm file or providing a valid existing path during test execution.
const DUMMY_PATH: &str = "fake-runtime-path";

const OUTPUT_FILE: &str = "/tmp/chain_spec_builder.test_output_file.json";

/// Asserts that the JSON in output file matches the JSON in expected file.
///
/// This helper function reads the JSON content from the file at `OUTPUT_FILE + suffix` path. If the
/// `overwrite_code` flag is set, it updates the output chain specification with a sample code
/// vector `[1, 2, 3]` (to avoid bulky *expected* files), and then compares it against the JSON
/// content from the given `expected_path`.
fn assert_output_eq_expected(overwrite_code: bool, output_suffix: &str, expected_path: &str) {
	let path = OUTPUT_FILE.to_string() + output_suffix;
	let mut output: serde_json::Value =
		serde_json::from_reader(File::open(path.clone()).unwrap()).unwrap();
	if overwrite_code {
		update_code_in_json_chain_spec(&mut output, &vec![1, 2, 3]);
	}
	let expected: serde_json::Value =
		serde_json::from_reader(File::open(expected_path).unwrap()).unwrap();

	assert_eq!(expected, output);

	std::fs::remove_file(path).expect("Failed to delete file");
}

fn get_builder(suffix: &str, command_args: Vec<&str>) -> ChainSpecBuilder {
	let path = OUTPUT_FILE.to_string() + suffix;
	let mut base_args = vec!["dummy", "-c", path.as_str()];
	base_args.extend(command_args);
	ChainSpecBuilder::parse_from(base_args)
}

#[test]
fn test_create_default() {
	const SUFFIX: &str = "00";
	let mut builder = get_builder(SUFFIX, vec!["create", "-r", DUMMY_PATH, "default"]);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_default.json");
}

#[test]
fn test_create_with_named_preset() {
	const SUFFIX: &str = "01";
	let mut builder =
		get_builder(SUFFIX, vec!["create", "-r", DUMMY_PATH, "named-preset", "staging"]);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_with_named_preset.json");
}

#[test]
fn test_create_with_patch() {
	const SUFFIX: &str = "02";
	let mut builder =
		get_builder(SUFFIX, vec!["create", "-r", DUMMY_PATH, "patch", "tests/input/patch.json"]);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_with_patch.json");
}

#[test]
fn test_create_with_full() {
	const SUFFIX: &str = "03";
	let mut builder =
		get_builder(SUFFIX, vec!["create", "-r", DUMMY_PATH, "full", "tests/input/full.json"]);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_with_full.json");
}

#[test]
fn test_create_with_params() {
	const SUFFIX: &str = "04";
	let mut builder = get_builder(
		SUFFIX,
		vec!["create", "-r", DUMMY_PATH, "-n", "test_chain", "-i", "100", "-t", "live", "default"],
	);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_with_params.json");
}

#[test]
fn test_create_parachain() {
	const SUFFIX: &str = "05";
	let mut builder = get_builder(
		SUFFIX,
		vec![
			"create",
			"-r",
			DUMMY_PATH,
			"-n",
			"test_chain",
			"-i",
			"100",
			"-t",
			"live",
			"--para-id",
			"10101",
			"--relay-chain",
			"rococo-local",
			"default",
		],
	);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_parachain.json");
}

#[test]
fn test_create_raw_storage() {
	const SUFFIX: &str = "06";
	let mut builder = get_builder(
		SUFFIX,
		vec!["create", "-r", DUMMY_PATH, "-s", "patch", "tests/input/patch.json"],
	);
	builder.set_create_cmd_runtime_code(substrate_test_runtime::WASM_BINARY.unwrap().into());
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/create_raw_storage.json");
}

#[test]
fn test_update_code() {
	const SUFFIX: &str = "07";
	let builder = get_builder(
		SUFFIX,
		vec!["update-code", "tests/input/chain_spec_plain.json", "tests/input/code_040506.blob"],
	);
	builder.run().unwrap();
	assert_output_eq_expected(false, SUFFIX, "tests/expected/update_code.json");
}

#[test]
fn test_update_code_raw() {
	const SUFFIX: &str = "08";
	let builder = get_builder(
		SUFFIX,
		vec!["update-code", "tests/input/chain_spec_raw.json", "tests/input/code_040506.blob"],
	);
	builder.run().unwrap();
	assert_output_eq_expected(false, SUFFIX, "tests/expected/update_code_raw.json");
}

#[test]
fn test_convert_to_raw() {
	const SUFFIX: &str = "09";
	let builder =
		get_builder(SUFFIX, vec!["convert-to-raw", "tests/input/chain_spec_conversion_test.json"]);
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/convert_to_raw.json");
}

#[test]
fn test_add_code_substitute() {
	const SUFFIX: &str = "10";
	let builder = get_builder(
		SUFFIX,
		vec![
			"add-code-substitute",
			"tests/input/chain_spec_plain.json",
			"tests/input/code_040506.blob",
			"100",
		],
	);
	builder.run().unwrap();
	assert_output_eq_expected(true, SUFFIX, "tests/expected/add_code_substitute.json");
}

#[docify::export]
#[test]
fn create_default() {
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let output_path = "/tmp/test.create_default.json";
	let expected_output = "tests/expected/doc/create_default.json";
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	run_cmd!(
		$bin_path -c $output_path create -r $runtime_path default;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn display_default_preset() {
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let output_path = "/tmp/test.display_preset.json";
	let expected_output = "tests/expected/doc/display_preset.json";
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	run_cmd!(
		$bin_path display-preset -r $runtime_path > $output_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn display_preset() {
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let preset_name = "staging";
	let output_path = format!("/tmp/test.display_preset_{preset_name}.json");
	let expected_output = format!("tests/expected/doc/display_preset_{preset_name}.json");
	run_cmd!(
		$bin_path display-preset -r $runtime_path -p $preset_name  > $output_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn list_presets() {
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let output_path = "/tmp/test.list_presets.json";
	let expected_output = "tests/expected/doc/list_presets.json";
	run_cmd!(
		$bin_path list-presets -r $runtime_path > $output_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn create_with_named_preset() {
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let named_preset = "staging";
	let output_path = format!("/tmp/test.create_with_named_preset_{named_preset}.json");
	let expected_output =
		format!("tests/expected/doc/create_with_named_preset_{named_preset}.json");
	run_cmd!(
		$bin_path -c $output_path create --relay-chain dev --para-id 1000 -r $runtime_path named-preset $named_preset;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn create_with_patch_raw() {
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let patch_path = "tests/input/patch.json";
	let output_path = format!("/tmp/test.create_with_patch_raw.json");
	let expected_output = format!("tests/expected/doc/create_with_patch_raw.json");
	run_cmd!(
		$bin_path -c $output_path create -s -r $runtime_path patch $patch_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn create_with_patch_plain() {
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let patch_path = "tests/input/patch.json";
	let output_path = format!("/tmp/test.create_with_patch_plain.json");
	let expected_output = format!("tests/expected/doc/create_with_patch_plain.json");
	run_cmd!(
		$bin_path -c $output_path create -r $runtime_path patch $patch_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn create_full_plain() {
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let full_genesis_config_path = "tests/input/full.json";
	let output_path = format!("/tmp/test.create_full_plain.json");
	let expected_output = format!("tests/expected/doc/create_full_plain.json");

	run_cmd!(
		$bin_path -c $output_path create -r $runtime_path full $full_genesis_config_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}

#[docify::export]
#[test]
fn create_full_raw() {
	let bin_path = env!("CARGO_BIN_EXE_chain-spec-builder");
	let runtime_path = substrate_test_runtime::WASM_BINARY_PATH.expect("a valid path. qed.");
	let full_genesis_config_path = "tests/input/full.json";
	let output_path = format!("/tmp/test.create_full_raw.json");
	let expected_output = format!("tests/expected/doc/create_full_raw.json");
	run_cmd!(
		$bin_path -c $output_path create -s -r $runtime_path full $full_genesis_config_path;
		sed -i "/\"code\":/d" $output_path;
		diff $output_path $expected_output;
		rm $output_path;
	)
	.expect("cmd success. qed.");
}
