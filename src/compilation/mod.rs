mod swaypad;
mod tooling;

use self::{
    swaypad::{create_project, remove_project, write_main_file},
    tooling::{build_project, check_forc_version, switch_fuel_toolchain},
};
use crate::{
    error::ApiError,
    types::CompileResponse,
    util::{clean_error_content, read_file_contents},
};
use hex::encode;
use std::fs::read_to_string;

const FILE_NAME: &str = "main.sw";

/// Build and destroy a project.
pub fn build_and_destroy_project(
    contract: String,
    toolchain: String,
) -> Result<CompileResponse, ApiError> {
    // Check if any contract has been submitted.
    if contract.is_empty() {
        return Ok(CompileResponse {
            abi: "".to_string(),
            bytecode: "".to_string(),
            storage_slots: "".to_string(),
            forc_version: "".to_string(),
            error: Some("No contract.".to_string()),
        });
    }

    // Switch to the given fuel toolchain and check forc version.
    switch_fuel_toolchain(toolchain);
    let forc_version = check_forc_version();

    // Create a new project.
    let project_name =
        create_project().map_err(|_| ApiError::Filesystem("create project".into()))?;

    // Write the file to the temporary project and compile.
    write_main_file(project_name.to_owned(), contract.as_bytes())
        .map_err(|_| ApiError::Filesystem("write main file".into()))?;
    let output = build_project(project_name.clone());

    // If the project compiled successfully, read the ABI and BIN files.
    if output.status.success() {
        let abi = read_to_string(format!(
            "projects/{project_name}/out/debug/swaypad-abi.json"
        ))
        .expect("Should have been able to read the file");
        let bin = read_file_contents(format!("projects/{project_name}/out/debug/swaypad.bin"));
        let storage_slots = read_file_contents(format!(
            "projects/{project_name}/out/debug/swaypad-storage_slots.json"
        ));

        // Remove the project directory and contents.
        remove_project(project_name).map_err(|_| ApiError::Filesystem("remove project".into()))?;

        // Return the abi, bin, empty error message, and forc version.
        Ok(CompileResponse {
            abi: clean_error_content(abi, FILE_NAME),
            bytecode: clean_error_content(encode(bin), FILE_NAME),
            storage_slots: String::from_utf8_lossy(&storage_slots).into(),
            forc_version,
            error: None,
        })
    } else {
        // Get the error message presented in the console output.
        let error = std::str::from_utf8(&output.stderr).unwrap();

        // Get the index of the main file.
        let main_index = error.find("/main.sw:").unwrap_or_default();

        // Truncate the error message to only include the relevant content.
        let trunc = String::from(error).split_off(main_index);

        // Remove the project.
        remove_project(project_name).unwrap();

        // Return an empty abi, bin, error message, and forc version.
        Ok(CompileResponse {
            abi: String::from(""),
            bytecode: String::from(""),
            storage_slots: String::from(""),
            error: Some(clean_error_content(trunc, FILE_NAME)),
            forc_version,
        })
    }
}
