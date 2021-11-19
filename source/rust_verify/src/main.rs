#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_mir_build;
extern crate rustc_span;
extern crate rustc_typeck;

use rust_verify::config;
use rust_verify::erase::CompilerCallbacks;
use rust_verify::verifier::Verifier;

#[cfg(target_family = "windows")]
fn os_setup() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Windows to kill the child SMT process if the parent is killed
    let job = win32job::Job::create()?;
    let mut info = job.query_extended_limit_info()?;
    info.limit_kill_on_job_close();
    job.set_extended_limit_info(&mut info)?;
    job.assign_current_process()?;
    // dropping the job object would kill us immediately, so just let it live forever instead:
    std::mem::forget(job);
    Ok(())
}

#[cfg(target_family = "unix")]
fn os_setup() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn mk_compiler<'a, 'b>(
    rustc_args: &'a [String],
    verifier: &'b mut (dyn rustc_driver::Callbacks + Send),
    pervasive_path: &Option<String>,
) -> rustc_driver::RunCompiler<'a, 'b> {
    let mut compiler = rustc_driver::RunCompiler::new(rustc_args, verifier);
    rust_verify::file_loader::PervasiveFileLoader::set_for_compiler(
        &mut compiler,
        pervasive_path.clone(),
    );
    compiler
}

pub fn main() {
    let _ = os_setup();

    let mut args = std::env::args();
    let program = args.next().unwrap();
    let (our_args, rustc_args) = config::parse_args(&program, args);
    let lifetime = our_args.lifetime;
    let compile = our_args.compile;
    let print_erased_spec = our_args.print_erased_spec;
    let print_erased = our_args.print_erased;
    let pervasive_path = our_args.pervasive_path.clone();

    // Run verifier callback to build VIR tree and run verifier

    let mut verifier = Verifier::new(our_args);

    let status = mk_compiler(&rustc_args, &mut verifier, &pervasive_path).run();
    if !verifier.encountered_vir_error {
        println!(
            "Verification results:: verified: {} errors: {}",
            verifier.count_verified,
            verifier.errors.len()
        );
    }
    match status {
        Ok(_) => {}
        Err(_) => {
            std::process::exit(1);
        }
    }

    // Run borrow checker with both #[code] and #[proof]
    if lifetime {
        let erasure_hints = verifier.erasure_hints.clone().expect("erasure_hints");
        let mut callbacks =
            CompilerCallbacks { erasure_hints, lifetimes_only: true, print: print_erased_spec };
        let status = mk_compiler(&rustc_args, &mut callbacks, &pervasive_path).run();
        match status {
            Ok(_) => {}
            Err(_) => {
                std::process::exit(1);
            }
        }
    }

    // Run borrow checker and compiler on #[code] (if enabled)
    if compile {
        let erasure_hints = verifier.erasure_hints.clone().expect("erasure_hints").clone();
        let mut callbacks =
            CompilerCallbacks { erasure_hints, lifetimes_only: false, print: print_erased };
        mk_compiler(&rustc_args, &mut callbacks, &pervasive_path)
            .run()
            .expect("RunCompiler.run() failed");
    }
}
