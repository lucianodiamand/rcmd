use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use std::str;

use clap::{Parser, Subcommand};

const HISTORY_FILE: &str = ".my_saved_history";

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Guarda el último comando ejecutado
    Save {
        #[arg(long)]
        tag: Option<String>,
    },
    /// Lista los comandos guardados
    List,
    /// Busca texto en los comandos guardados
    Grep {
        texto: String,
    },
    /// Ejecuta el comando N
    Exec {
        numero: usize,
    },
    /// Elimina el comando N
    Rm {
        numero: usize,
    },
    /// Muestra el comando N
    Print {
        numero: usize,
    },
}

fn history_path() -> PathBuf {
    dirs::home_dir().unwrap().join(HISTORY_FILE)
}

fn read_history() -> Vec<String> {
    let path = history_path();
    if path.exists() {
        let file = fs::File::open(path).expect("No se pudo abrir el archivo de historial");
        BufReader::new(file)
            .lines()
            .filter_map(Result::ok)
            .collect()
    } else {
        vec![]
    }
}

fn write_history(lines: &[String]) {
    let mut file = fs::File::create(history_path()).expect("No se pudo escribir historial");
    for line in lines {
        writeln!(file, "{}", line).unwrap();
    }
}

fn save_command(tag: Option<String>) {
    let shell_history = dirs::home_dir().unwrap().join(".zsh_history");
    let content = fs::read_to_string(&shell_history).unwrap_or_default();
    let history_lines: Vec<_> = content.lines().collect();

    let last_cmd = history_lines
        .iter()
        .rev()
        .filter_map(|line| {
            if line.starts_with(": ") {
                let parts: Vec<&str> = line.splitn(2, ';').collect();
                if parts.len() == 2 {
                    let cmd = parts[1].trim();
                    if !cmd.starts_with("r ") && !cmd.contains("rcmd save") {
                        return Some(cmd.to_string());
                    }
                }
            }
            None
        })
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("No se encontró un comando anterior válido.");
            exit(1);
        });

    let tagged = format!("{} || {}", last_cmd, tag.unwrap_or_default());
    let mut history = read_history();

    if !history.iter().any(|l| l == &tagged) {
        history.push(tagged);
        write_history(&history);
    }
}

fn list_commands() {
    for (i, line) in read_history().iter().enumerate() {
        let parts: Vec<&str> = line.splitn(2, "||").collect();
        let cmd = parts[0].trim();
        let tag = parts.get(1).map(|s| s.trim()).unwrap_or("");
        if tag.is_empty() {
            println!("{:2}: {}", i + 1, cmd);
        } else {
            println!("{:2}: {} [tag: {}]", i + 1, cmd, tag);
        }
    }
}

fn grep_commands(pat: &str) {
    for (i, line) in read_history().iter().enumerate() {
        if line.contains(pat) {
            println!("{:2}: {}", i + 1, line);
        }
    }
}

fn exec_command(n: usize) {
    let cmds = read_history();
    if let Some(line) = cmds.get(n - 1) {
        let cmd = line.splitn(2, "||").next().unwrap().trim();
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .expect("Fallo al ejecutar el comando");
        if !status.success() {
            exit(status.code().unwrap_or(1));
        }
    } else {
        eprintln!("No existe el comando N={}" , n);
        exit(1);
    }
}

fn remove_command(n: usize) {
    let mut cmds = read_history();
    if n == 0 || n > cmds.len() {
        eprintln!("Número inválido");
        exit(1);
    }
    cmds.remove(n - 1);
    write_history(&cmds);
}

fn print_command(n: usize) {
    let cmds = read_history();
    if let Some(line) = cmds.get(n - 1) {
        println!("{}", line.splitn(2, "||").next().unwrap().trim());
    } else {
        eprintln!("No existe el comando N={}", n);
        exit(1);
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Save { tag }) => save_command(tag),
        Some(Commands::List) => list_commands(),
        Some(Commands::Grep { texto }) => grep_commands(&texto),
        Some(Commands::Exec { numero }) => exec_command(numero),
        Some(Commands::Rm { numero }) => remove_command(numero),
        Some(Commands::Print { numero }) => print_command(numero),
        None => save_command(None),
    }
}

