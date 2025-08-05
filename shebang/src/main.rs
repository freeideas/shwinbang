use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn main() {
    // 1. Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: shebang <script> [args]");
        std::process::exit(1);
    }

    let script_path = &args[1];
    let script_args = &args[2..];

    // 2. Read the first line of the script
    let file = match File::open(script_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: Failed to open script file '{}': {}", script_path, e);
            std::process::exit(1);
        }
    };
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    if let Err(e) = reader.read_line(&mut first_line) {
        eprintln!("Error: Failed to read from script file '{}': {}", script_path, e);
        std::process::exit(1);
    }

    // 3. Parse the shebang line
    if !first_line.starts_with("#!") {
        eprintln!("Error: No shebang line found in '{}'.", script_path);
        std::process::exit(1);
    }

    let shebang_cmd = first_line.trim_start_matches("#!").trim();
    let mut cmd_parts: Vec<&str> = shebang_cmd.split_whitespace().collect();

    // Handle /usr/bin/env on Windows by stripping it and its options
    if cfg!(windows) && cmd_parts.get(0) == Some(&"/usr/bin/env") {
        cmd_parts.remove(0); // remove /usr/bin/env
        if !cmd_parts.is_empty() && (cmd_parts[0] == "-S" || cmd_parts[0] == "-") {
            cmd_parts.remove(0);
        }
    }

    if cmd_parts.is_empty() {
        eprintln!("Error: Invalid shebang line in '{}'.", script_path);
        std::process::exit(1);
    }

    let executable = cmd_parts[0];
    let mut shebang_args = Vec::new();
    let mut script_first = true;

    for (i, part) in cmd_parts.iter().skip(1).enumerate() {
        if *part == "--" {
            script_first = false;
            shebang_args.extend(cmd_parts.iter().skip(i + 2));
            break;
        }
        shebang_args.push(*part);
    }


    // 4. Construct and execute the final command
    let mut final_command = Command::new(executable);

    if script_first {
        final_command.args(&shebang_args);
        final_command.arg(script_path);
        final_command.args(script_args);
    } else {
        final_command.args(&shebang_args);
        final_command.args(script_args);
        final_command.arg(script_path);
    }


    let mut child = match final_command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: Failed to execute command: '{}': {}", shebang_cmd, e);
            std::process::exit(1);
        }
    };

    // 5. Wait for the command to complete and exit with its status
    match child.wait() {
        Ok(status) => {
            if let Some(code) = status.code() {
                std::process::exit(code);
            } else {
                eprintln!("Process terminated by signal");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to wait for child process: {}", e);
            std::process::exit(1);
        }
    }
}
