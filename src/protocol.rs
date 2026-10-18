use crate::device::{DATA_REPORT_LEN, Mouse, STEP_DELAY};
use crate::general::General;
use anyhow::{Result, bail};
use std::thread::sleep;
use std::time::Duration;

pub const BLOCK_OFFSET: usize = 8;
pub const GENERAL_LEN: usize = 0x8C;
pub const POLLING_BYTE: usize = 2;
pub const SLOT_COUNT: usize = 20;
const BUTTONS_LEN: u8 = 0x50;
const WRITE_SETTLE: Duration = Duration::from_millis(200);

pub type ButtonMap = [u32; SLOT_COUNT];

fn selector(profile: u8, block: u8) -> u8 {
    profile << 4 | block
}

fn read_block(mouse: &Mouse, profile: u8, block: u8) -> Result<[u8; DATA_REPORT_LEN]> {
    let sel = selector(profile, block);
    let data = mouse.query_data([sel, 0, 0, 0, 0])?;
    if data[1] != sel {
        bail!("resposta inesperada do perfil {profile}: {:02x}", data[1]);
    }
    Ok(data)
}

fn write_block(mouse: &Mouse, profile: u8, block: u8, len: u8, body: &[u8]) -> Result<()> {
    let mut payload = vec![selector(profile, block), 0, len, 0, 0, 0, 0];
    payload.extend_from_slice(body);
    sleep(STEP_DELAY);
    mouse.send_data(&payload)?;
    sleep(WRITE_SETTLE);
    Ok(())
}

pub fn read_general(mouse: &Mouse, profile: u8) -> Result<[u8; DATA_REPORT_LEN]> {
    read_block(mouse, profile, 0x01)
}

pub fn write_general(mouse: &Mouse, profile: u8, block: &[u8]) -> Result<()> {
    write_block(mouse, profile, 0x01, 0x7B, block)
}

pub fn read_buttons(mouse: &Mouse, profile: u8) -> Result<ButtonMap> {
    let data = read_block(mouse, profile, 0x02)?;
    let mut map = [0u32; SLOT_COUNT];
    for (i, slot) in map.iter_mut().enumerate() {
        let at = BLOCK_OFFSET + i * 4;
        *slot = u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    }
    Ok(map)
}

pub fn write_buttons(mouse: &Mouse, profile: u8, map: &ButtonMap) -> Result<ButtonMap> {
    if !map.contains(&crate::buttons::Action::LEFT_CLICK) {
        bail!("pelo menos um botão tem de ficar com o clique esquerdo");
    }
    let body: Vec<u8> = map.iter().flat_map(|v| v.to_le_bytes()).collect();
    write_block(mouse, profile, 0x02, BUTTONS_LEN, &body)?;
    let after = read_buttons(mouse, profile)?;
    if &after != map {
        bail!("o mapa lido depois da escrita não corresponde ao escrito");
    }
    Ok(after)
}

pub fn read_settings(mouse: &Mouse, profile: u8) -> Result<General> {
    let data = read_general(mouse, profile)?;
    Ok(General::from_slice(&data[BLOCK_OFFSET..]))
}

pub fn write_settings(mouse: &Mouse, profile: u8, settings: &General) -> Result<General> {
    write_general(mouse, profile, &settings.0)?;
    let after = read_settings(mouse, profile)?;
    let diff = after.diff(settings);
    if !diff.is_empty() {
        let at: Vec<String> = diff.iter().map(|i| format!("{i:#04x}")).collect();
        bail!("o bloco lido depois da escrita difere nos bytes {}", at.join(", "));
    }
    Ok(after)
}

pub fn active_profile(mouse: &Mouse) -> Result<u8> {
    let reply = mouse.query([0x02, 0, 0, 0, 0])?;
    if reply[0] != 0x02 || !(1..=3).contains(&reply[1]) {
        bail!("resposta inesperada ao pedir o perfil activo: {reply:02x?}");
    }
    Ok(reply[1])
}

pub fn activate_profile(mouse: &Mouse, profile: u8) -> Result<()> {
    if !(1..=3).contains(&profile) {
        bail!("perfil inválido: {profile}");
    }
    sleep(STEP_DELAY);
    mouse.send_cmd([0x02, profile, 0, 0, 0])?;
    sleep(WRITE_SETTLE);
    Ok(())
}

pub fn polling_code(hz: u16) -> Result<u8> {
    Ok(match hz {
        125 => 1,
        250 => 2,
        500 => 3,
        1000 => 4,
        _ => bail!("taxa inválida: use 125, 250, 500 ou 1000"),
    })
}

pub fn available_profiles(mouse: &Mouse) -> Vec<u8> {
    (1..=3u8).filter(|&p| read_general(mouse, p).is_ok()).collect()
}
