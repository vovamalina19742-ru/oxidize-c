use clap::{Parser, Subcommand};
use colored::*;
use oxidize_core::{CSourceAnalyzer, SafetyScoreReport};
use oxidize_llm::LlmOrchestrator;
use oxidize_verifier::CompilerVerifier;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxidize-c")]
#[command(about = "🚀 Intelligent Hybrid C-to-Safe-Rust Transpiler & Verification Engine (Engineering Prototype)", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze C source code and output structural AST memory pattern classifications
    Analyze {
        /// Path to the C source file
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Transpile a C file into idiomatic Safe Rust with compiler verification
    Translate {
        /// Path to C source file
        #[arg(short, long)]
        input: PathBuf,
        /// Output path for generated Safe Rust code (defaults to input with .rs extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Verify compilation with rustc (--emit=metadata)
        #[arg(short, long, default_value_t = true)]
        verify: bool,
        /// Run internal unit tests if present (rustc --test)
        #[arg(short = 't', long, default_value_t = false)]
        run_tests: bool,
        /// Run Miri dynamic verification if available
        #[arg(short = 'm', long, default_value_t = false)]
        miri: bool,
    },
    /// Calculate Safety Score for a translated Rust codebase
    Score {
        /// Original C file
        #[arg(short, long)]
        c_file: PathBuf,
        /// Translated Rust file
        #[arg(short, long)]
        rust_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { file } => {
            if !file.exists() {
                anyhow::bail!("Input file does not exist: {:?}", file);
            }
            println!("{}", "=== OXIDIZE-C STATIC AST ANALYZER ===".bright_cyan().bold());
            let source = std::fs::read_to_string(&file)?;
            let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let analyzer = CSourceAnalyzer::new(&source, filename);
            let report = analyzer.analyze();

            println!("{}: {}", "Target File".bright_yellow(), report.filename);
            println!("{}: {}", "Total Lines".bright_yellow(), report.total_lines);
            println!("{}: {}", "Dynamic Memory (malloc/free)".bright_yellow(), report.has_malloc);
            println!("{}: {}", "OS Descriptors (Socket/FD)".bright_yellow(), report.has_os_socket);

            println!("\n{}", "--- Structural Classifications ---".bright_cyan());
            for s in &report.structs {
                println!("  • struct {}: {}", s.name.bold(), format!("{:?}", s.pattern).green());
            }

            println!("\n{}", "--- Mandatory Directives ---".bright_cyan());
            for p in &report.suggested_patterns {
                println!("  ✔ {}", p.description().green());
            }
        }
        Commands::Translate {
            input,
            output,
            verify,
            run_tests,
            miri,
        } => {
            if !input.exists() {
                anyhow::bail!("Input C file does not exist: {:?}", input);
            }

            println!("{}", "=== OXIDIZE-C TRANSPILATION PIPELINE ===".bright_green().bold());
            println!("Input: {:?}", input);
            let out_path = output.unwrap_or_else(|| input.with_extension("rs"));
            println!("Output: {:?}", out_path);

            let c_code = std::fs::read_to_string(&input)?;
            let filename = input.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let analyzer = CSourceAnalyzer::new(&c_code, filename);
            let report = analyzer.analyze();

            println!("\n[*] Step 1: AST Analysis complete ({} lines, {} structural patterns detected)",
                report.total_lines,
                report.suggested_patterns.len()
            );

            // Attempt translation via configured LLM orchestrator
            let orchestrator = match LlmOrchestrator::from_env() {
                Ok(orch) => Some(orch),
                Err(_) => {
                    println!("{}", "[!] Warning: No LLM API key detected (GEMINI_API_KEY, OPENROUTER_API_KEY, or OLLAMA_HOST).".bright_yellow());
                    None
                }
            };

            let mut rust_code = String::new();
            if let Some(orch) = orchestrator {
                println!("[*] Step 2: Querying LLM provider with AST directives...");
                let mut compiler_err: Option<String> = None;
                const MAX_ATTEMPTS: usize = 3;

                for attempt in 1..=MAX_ATTEMPTS {
                    println!("    Attempt {}/{}...", attempt, MAX_ATTEMPTS);
                    let translated = orch
                        .transpile(&c_code, &report, compiler_err.as_deref())
                        .await?;

                    if verify {
                        match CompilerVerifier::verify_syntax(&translated, &out_path) {
                            Ok(()) => {
                                println!("{}", "    [+] Compiler syntax & borrow check PASSED.".bright_green());
                                rust_code = translated;
                                break;
                            }
                            Err(e) => {
                                println!("{}", format!("    [-] Compiler check failed: {}", e).bright_red());
                                compiler_err = Some(e.to_string());
                                if attempt == MAX_ATTEMPTS {
                                    anyhow::bail!("Transpilation self-healing loop failed after {} attempts:\n{}", MAX_ATTEMPTS, e);
                                }
                            }
                        }
                    } else {
                        rust_code = translated;
                        break;
                    }
                }
            } else if out_path.exists() {
                println!("{}", "[*] Step 2: Using existing reference code for verification.".bright_cyan());
                rust_code = std::fs::read_to_string(&out_path)?;
            } else {
                anyhow::bail!(
                    "Cannot translate: No LLM provider configured and no output reference exists.\nPlease set GEMINI_API_KEY, OPENROUTER_API_KEY, or OLLAMA_HOST."
                );
            }

            // Write output
            std::fs::write(&out_path, &rust_code)?;
            println!("[+] Output written to: {:?}", out_path);

            // Step 3: Verification
            if verify {
                println!("\n[*] Step 3: Running Comprehensive Verification Suite...");
                let verif_report = CompilerVerifier::verify_full(&rust_code, &out_path, run_tests, miri)?;
                for diag in &verif_report.diagnostics {
                    println!("    • {}", diag);
                }
            }

            // Step 4: Safety Score calculation
            let score_report = SafetyScoreReport::compute(&c_code, &rust_code);
            println!("\n{}", "=== SAFETY SCORE AUDIT ===".bright_cyan());
            println!("Safety Score: {}%", format!("{:.1}", score_report.safety_score_percentage).bright_green().bold());
            println!("Unsafe Blocks: {}", score_report.unsafe_blocks.to_string().bright_green());
            println!("Raw Pointers: {}", score_report.raw_pointers_count.to_string().bright_green());

            println!("\n{}", "[✔] Transpilation and verification pipeline completed successfully!".bright_green().bold());
        }
        Commands::Score { c_file, rust_file } => {
            if !c_file.exists() {
                anyhow::bail!("C file does not exist: {:?}", c_file);
            }
            if !rust_file.exists() {
                anyhow::bail!("Rust file does not exist: {:?}", rust_file);
            }
            let c_code = std::fs::read_to_string(&c_file)?;
            let rust_code = std::fs::read_to_string(&rust_file)?;

            let report = SafetyScoreReport::compute(&c_code, &rust_code);

            println!("{}", "=================================================".bright_cyan());
            println!("{}", "           OXIDIZE-C SAFETY SCORE REPORT         ".bright_white().bold());
            println!("{}", "=================================================".bright_cyan());
            println!("Safety Score: {}%", format!("{:.1}", report.safety_score_percentage).bright_green().bold());
            println!("Unsafe Blocks: {}", report.unsafe_blocks.to_string().bright_green());
            println!("Raw Pointers: {}", report.raw_pointers_count.to_string().bright_green());
            println!("Memory Leak Risk: {}", report.memory_leak_risk.bright_green());
            println!("Data Race Risk: {}", report.data_race_risk.bright_green());
            println!("{}", "=================================================".bright_cyan());
        }
    }

    Ok(())
}
