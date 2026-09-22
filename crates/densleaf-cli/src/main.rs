use std::{env, fs, path::Path, process::ExitCode};

use densleaf_codegen::generate_rust;
use densleaf_diagnostic::Diagnostic;
use densleaf_lexer::lex;
use densleaf_parser::parse;
use densleaf_semantic::analyze;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}

fn run(args: Vec<String>) -> Result<(), u8> {
    if args.len() != 2 || !matches!(args[0].as_str(), "check" | "build") {
        eprintln!("usage: densleaf <check|build> <file>");
        return Err(2);
    }

    let command = &args[0];
    let path = Path::new(&args[1]);
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: could not read {}: {error}", path.display());
            return Err(1);
        }
    };
    let file = path.display().to_string();

    let lexed = lex(&source, file);
    if print_diagnostics(&lexed.diagnostics, &source) {
        return Err(1);
    }

    let parsed = parse(lexed.tokens);
    if print_diagnostics(&parsed.diagnostics, &source) {
        return Err(1);
    }

    let semantic_diagnostics = analyze(&parsed.program);
    if print_diagnostics(&semantic_diagnostics, &source) {
        return Err(1);
    }

    if command == "check" {
        println!("Densleaf check passed: {}", path.display());
        return Ok(());
    }

    let generated = generate_rust(&parsed.program);
    let output_dir = Path::new(".densleaf/generated");
    if let Err(error) = fs::create_dir_all(output_dir) {
        eprintln!("error: could not create {}: {error}", output_dir.display());
        return Err(1);
    }
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("app");
    let output_path = output_dir.join(format!("{stem}.rs"));
    if let Err(error) = fs::write(&output_path, generated) {
        eprintln!("error: could not write {}: {error}", output_path.display());
        return Err(1);
    }

    println!("Generated Rust: {}", output_path.display());
    Ok(())
}

fn print_diagnostics(diagnostics: &[Diagnostic], source: &str) -> bool {
    if diagnostics.is_empty() {
        return false;
    }

    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            eprintln!();
        }
        eprintln!("{}", diagnostic.render(source));
    }
    true
}
