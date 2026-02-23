use crate::config::VerifyRequest;
use crate::error::NszError;
use crate::ops::VerifyReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &VerifyRequest) -> Result<VerifyReport, NszError> {
    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut verified_files = Vec::new();

    for file in &request.files {
        let mut args = vec!["-V".to_string()];
        if request.fix_padding {
            args.push("-F".to_string());
        }
        args.push(file.display().to_string());

        run_nsz_cli(&repo_root, &args)?;
        verified_files.push(file.clone());
    }

    Ok(VerifyReport { verified_files })
}
