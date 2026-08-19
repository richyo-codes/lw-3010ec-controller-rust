use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::path::PathBuf;
use std::time::Duration;

// Reedline imports
use reedline::{
    DefaultCompleter, DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Reedline, Signal,
};

mod modbus;
mod psu;
mod serial;

#[derive(Parser, Debug)]
#[command(name = "lw3010ec-controller")]
#[command(about = "Topshak/Longwei LW-3010EC bench power supply controller")]
struct Cli {
    /// Serial port to use (e.g. /dev/ttyUSB0)
    #[arg(short, long)]
    port: Option<String>,

    /// Modbus unit/slave ID (default: 1)
    #[arg(short, long, default_value = "1")]
    unit_id: u8,

    /// Response timeout in seconds (default: 3)
    #[arg(short, long, default_value = "3")]
    timeout: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan for connected PSUs
    Scan,

    /// Read PSU status
    Status,

    /// Read output voltage
    GetVoltage,

    /// Read current limit setting
    GetCurrent,

    /// Read output state
    GetOutput,

    /// Set voltage (0-30V)
    SetVoltage {
        /// Voltage value (0–30V)
        value: f32,
    },

    /// Set current limit (0-10A)
    SetCurrent {
        /// Current limit value (0–10A)
        value: f32,
    },

    /// Turn output on
    On,

    /// Turn output off
    Off,

    /// Start interactive REPL
    Repl,

    /// Generate shell completions for bash, zsh, fish, elvish, or powershell
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, elvish, powershell)
        shell: Shell,

        /// Output directory (default: prints to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build the WebAssembly module (requires wasm-pack)
    Wasm {
        /// Output directory (default: web/pkg)
        #[arg(long, default_value = "web/pkg")]
        out_dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let timeout = Duration::from_secs(cli.timeout);

    match cli.command {
        Commands::Scan => {
            let ports = serial::find_psu_ports()?;
            if ports.is_empty() {
                println!("No PSU found. Try specifying port manually with --port");
                println!("Available serial ports:");
                for port in serial::list_ports()? {
                    println!("  {}", port.port_name);
                }
            } else {
                println!("Found {} PSU(s):", ports.len());
                for port in &ports {
                    println!("  {}", port.port_name);
                }
            }
        }
        Commands::Status => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::get_status(cli.unit_id, &mut *port, timeout) {
                Ok(status) => println!("{}", status),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GetVoltage => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::get_voltage(cli.unit_id, &mut *port, timeout) {
                Ok(v) => println!("{:.2}V", v),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GetCurrent => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::get_current(cli.unit_id, &mut *port, timeout) {
                Ok(c) => println!("{:.3}A", c),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::GetOutput => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::get_output(cli.unit_id, &mut *port, timeout) {
                Ok(on) => println!("{}", if on { "ON" } else { "OFF" }),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::SetVoltage { value } => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::set_voltage(cli.unit_id, &mut *port, value, timeout) {
                Ok(()) => println!("Voltage set to {:.2}V", value),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::SetCurrent { value } => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::set_current(cli.unit_id, &mut *port, value, timeout) {
                Ok(()) => println!("Current set to {:.3}A", value),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::On => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::set_output(cli.unit_id, &mut *port, true, timeout) {
                Ok(()) => println!("Output enabled"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Off => {
            let port_name = cli.port.ok_or_else(|| {
                anyhow::anyhow!("Port not specified. Use --port or run 'scan' first")
            })?;
            let mut port = serial::open_port(&port_name, 9600)?;
            match psu::PsuController::set_output(cli.unit_id, &mut *port, false, timeout) {
                Ok(()) => println!("Output disabled"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Repl => {
            let port_name = cli
                .port
                .ok_or_else(|| anyhow::anyhow!("Port not specified. Use --port"))?;
            let mut port = serial::open_port(&port_name, 9600)?;
            println!("LW-3010EC REPL (type 'help' for commands, 'exit' to quit)");
            println!();
            run_repl(cli.unit_id, &mut *port, timeout);
        }
        Commands::Completions { shell, output } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            if let Some(dir) = output {
                std::fs::create_dir_all(&dir)?;
                let mut file_path = dir.clone();
                file_path.push(format!("_{}", bin_name));
                let mut file = std::fs::File::create(file_path)?;
                generate(shell, &mut cmd, &bin_name, &mut file);
            } else {
                generate(shell, &mut cmd, &bin_name, &mut std::io::stdout());
            }
        }
        Commands::Wasm { out_dir } => {
            println!("Building WASM module for web UI...");
            let status = std::process::Command::new("wasm-pack")
                .args([
                    "build",
                    "--target",
                    "web",
                    "--out-name",
                    "lw3010ec_core",
                    "--out-dir",
                    &out_dir,
                    "lw3010ec-core",
                ])
                .status()?;
            if status.success() {
                println!("WASM module built → {}/", out_dir);
            } else {
                eprintln!("wasm-pack build failed. Install with: cargo install wasm-pack");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn run_repl(unit_id: u8, port: &mut dyn serialport::SerialPort, timeout: Duration) {
    println!("LW-3010EC REPL (type 'help' for commands, 'exit' to quit)");
    println!();

    // Setup command history
    let history = FileBackedHistory::with_file(50, PathBuf::from(".psu_repl_history"))
        .expect("Error configuring history");

    // Setup tab completion
    let commands = vec![
        String::from("status"),
        String::from("get-voltage"),
        String::from("get-current"),
        String::from("get-output"),
        String::from("set-voltage"),
        String::from("set-current"),
        String::from("on"),
        String::from("off"),
        String::from("help"),
        String::from("exit"),
        String::from("quit"),
        String::from("q"),
    ];
    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 3));

    // Setup the Reedline engine
    let mut engine = Reedline::create()
        .with_completer(completer)
        .with_history(Box::new(history));

    // Main REPL loop
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("psu> ".to_string()),
        DefaultPromptSegment::Basic("".to_string()),
    );
    loop {
        let sig = engine.read_line(&prompt);

        match sig {
            Ok(Signal::Success(line)) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // Handle exit/quit
                if matches!(line.as_str(), "exit" | "quit" | "q") {
                    break;
                }

                // Parse command and arguments
                let parts: Vec<&str> = line.split_whitespace().collect();
                let cmd = parts[0].to_lowercase();

                let result = match cmd.as_str() {
                    "help" => {
                        println!("Available commands:");
                        println!("  status            - Read all PSU status");
                        println!("  get-voltage       - Read output voltage");
                        println!("  get-current       - Read current limit");
                        println!("  get-output        - Read output state");
                        println!("  set-voltage <V>   - Set voltage (0-30V)");
                        println!("  set-current <A>   - Set current limit (0-10A)");
                        println!("  on                - Turn output on");
                        println!("  off               - Turn output off");
                        println!("  help              - Show this help");
                        println!("  exit / quit / q   - Exit REPL");
                        continue;
                    }
                    "get-voltage" => {
                        match psu::PsuController::get_voltage(unit_id, port, timeout) {
                            Ok(v) => Some(format!("{:.2}V", v)),
                            Err(e) => Some(format!("Error: {}", e)),
                        }
                    }
                    "get-current" => {
                        match psu::PsuController::get_current(unit_id, port, timeout) {
                            Ok(c) => Some(format!("{:.3}A", c)),
                            Err(e) => Some(format!("Error: {}", e)),
                        }
                    }
                    "get-output" => match psu::PsuController::get_output(unit_id, port, timeout) {
                        Ok(on) => Some(if on { "ON".into() } else { "OFF".into() }),
                        Err(e) => Some(format!("Error: {}", e)),
                    },
                    "status" => match psu::PsuController::get_status(unit_id, port, timeout) {
                        Ok(s) => Some(format!("{}", s)),
                        Err(e) => Some(format!("Error: {}", e)),
                    },
                    "set-voltage" => {
                        let value: f32 = match parts.get(1).and_then(|v| v.parse().ok()) {
                            Some(v) => v,
                            None => {
                                println!("Usage: set-voltage <value 0-30>");
                                continue;
                            }
                        };
                        match psu::PsuController::set_voltage(unit_id, port, value, timeout) {
                            Ok(()) => Some(format!("Voltage set to {:.2}V", value)),
                            Err(e) => Some(format!("Error: {}", e)),
                        }
                    }
                    "set-current" => {
                        let value: f32 = match parts.get(1).and_then(|v| v.parse().ok()) {
                            Some(v) => v,
                            None => {
                                println!("Usage: set-current <value 0-10>");
                                continue;
                            }
                        };
                        match psu::PsuController::set_current(unit_id, port, value, timeout) {
                            Ok(()) => Some(format!("Current set to {:.3}A", value)),
                            Err(e) => Some(format!("Error: {}", e)),
                        }
                    }
                    "on" => match psu::PsuController::set_output(unit_id, port, true, timeout) {
                        Ok(()) => Some("Output enabled".to_string()),
                        Err(e) => Some(format!("Error: {}", e)),
                    },
                    "off" => match psu::PsuController::set_output(unit_id, port, false, timeout) {
                        Ok(()) => Some("Output disabled".to_string()),
                        Err(e) => Some(format!("Error: {}", e)),
                    },
                    _ => {
                        println!(
                            "Unknown command: '{}'. Type 'help' for available commands.",
                            parts[0]
                        );
                        continue;
                    }
                };

                if let Some(resp) = result {
                    println!("{}", resp);
                }
            }
            Ok(Signal::CtrlD) => break,    // EOF
            Ok(Signal::CtrlC) => continue, // Ignore Ctrl+C
            Err(e) => {
                println!("Reedline error: {:?}", e);
                break;
            }
        }
    }

    println!();
}
