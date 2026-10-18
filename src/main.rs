use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use gxt970::buttons::{Action, BUTTONS};
use gxt970::device::Mouse;
use gxt970::protocol::{self, BLOCK_OFFSET, GENERAL_LEN, POLLING_BYTE};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gxt970", about = "Configuração do rato Trust GXT 970")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Probe,
    Raw { sel: String },
    Backup { file: PathBuf },
    Polling { profile: u8, hz: u16 },
    Buttons { profile: u8 },
    RestoreDpi { profile: u8, backup: PathBuf },
    Activate { profile: u8 },
    Dpi { profile: u8, level: usize, value: u16 },
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .chunks(16)
        .enumerate()
        .map(|(i, c)| {
            let line: Vec<String> = c.iter().map(|b| format!("{b:02x}")).collect();
            format!("{:04x}: {}", i * 16, line.join(" "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mouse = Mouse::open()?;
    match cli.command {
        Command::Raw { sel } => {
            let sel = u8::from_str_radix(&sel, 16)?;
            let data = mouse.query_data([sel, 0, 0, 0, 0])?;
            println!("{}", hex(&data[..0x60]));
        }
        Command::Probe => {
            println!("PSD:  {:02x?}", mouse.query([1, 0, 0, 0, 0])?);
            println!("Modo: {:02x?}", mouse.query([2, 0, 0, 0, 0])?);
            for p in 1..=3u8 {
                match protocol::read_general(&mouse, p) {
                    Ok(data) => println!("\nPerfil {p}:\n{}", hex(&data[..0xA0])),
                    Err(e) => println!("\nPerfil {p}: sem resposta ({e})"),
                }
            }
        }
        Command::Buttons { profile } => {
            let map = protocol::read_buttons(&mouse, profile)?;
            for b in &BUTTONS {
                println!("{:<4} {:<18} {}", b.badge, b.name, Action::decode(map[b.slot - 1]).label());
            }
        }
        Command::Dpi { profile, level, value } => {
            if !(1..=gxt970::general::DPI_LEVELS).contains(&level) || gxt970::general::code_from_dpi(value).is_none() {
                bail!("nível 1 a 6 e DPI da tabela (200 a 10000)");
            }
            let mut g = protocol::read_settings(&mouse, profile)?;
            g.set_level_enabled(level - 1, true);
            g.set_dpi(level - 1, value);
            g.set_current_level(level - 1);
            let after = protocol::write_settings(&mouse, profile, &g)?;
            protocol::activate_profile(&mouse, profile)?;
            let dpis: Vec<String> = (0..6).filter_map(|l| after.dpi(l)).map(|d| d.to_string()).collect();
            println!("Níveis: {} | em uso: {:?}", dpis.join(", "), after.dpi(after.current_level()));
        }
        Command::Activate { profile } => {
            protocol::activate_profile(&mouse, profile)?;
            println!("Perfil activo: {}", protocol::active_profile(&mouse)?);
        }
        Command::RestoreDpi { profile, backup } => {
            let bytes = std::fs::read(&backup)?;
            let at = (profile as usize - 1) * 0x208 + BLOCK_OFFSET;
            if bytes.len() < at + GENERAL_LEN {
                bail!("o backup não tem o perfil {profile}");
            }
            let saved = gxt970::general::General::from_slice(&bytes[at..]);
            let mut current = protocol::read_settings(&mouse, profile)?;
            current.0[3..0x2D].copy_from_slice(&saved.0[3..0x2D]);
            let after = protocol::write_settings(&mouse, profile, &current)?;
            let dpis: Vec<String> = (0..6).filter_map(|l| after.dpi(l)).map(|d| d.to_string()).collect();
            println!("Níveis: {} | em uso: {:?}", dpis.join(", "), after.dpi(after.current_level()));
        }
        Command::Polling { profile, hz } => {
            if !(1..=3).contains(&profile) {
                bail!("perfil inválido: use 1, 2 ou 3");
            }
            let code = protocol::polling_code(hz)?;
            let before = protocol::read_general(&mouse, profile)?;
            let mut block = before[BLOCK_OFFSET..BLOCK_OFFSET + GENERAL_LEN].to_vec();
            let old = block[POLLING_BYTE];
            block[POLLING_BYTE] = old & 0x80 | code;
            println!("Perfil {profile}: byte de taxa {old:02x} -> {:02x}", block[POLLING_BYTE]);
            protocol::write_general(&mouse, profile, &block)?;
            let after = protocol::read_general(&mouse, profile)?;
            let new = &after[BLOCK_OFFSET..BLOCK_OFFSET + GENERAL_LEN];
            let diffs: Vec<String> = (0..GENERAL_LEN)
                .filter(|&i| new[i] != before[BLOCK_OFFSET + i])
                .map(|i| format!("{i:#04x}: {:02x} -> {:02x}", before[BLOCK_OFFSET + i], new[i]))
                .collect();
            println!("Bytes alterados no rato: {diffs:?}");
            if new != block.as_slice() {
                bail!("o bloco lido depois da escrita não corresponde ao escrito");
            }
            println!("Taxa do perfil {profile} definida para {hz} Hz");
        }
        Command::Backup { file } => {
            let mut out = Vec::new();
            let mut saved = Vec::new();
            for p in 1..=3u8 {
                match protocol::read_general(&mouse, p) {
                    Ok(data) => {
                        out.extend_from_slice(&data);
                        saved.push(p.to_string());
                    }
                    Err(e) => eprintln!("Perfil {p} ignorado: {e}"),
                }
            }
            if saved.is_empty() {
                bail!("nenhum perfil foi lido");
            }
            std::fs::write(&file, &out)?;
            println!("Cópia dos perfis {} gravada em {}", saved.join(", "), file.display());
        }
    }
    Ok(())
}
