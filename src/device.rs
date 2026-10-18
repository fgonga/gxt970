use anyhow::{Context, Result, anyhow, bail};
use hidapi::{HidApi, HidDevice};
use std::thread::sleep;
use std::time::Duration;

pub const VENDOR_ID: u16 = 0x145F;
pub const PRODUCT_ID: u16 = 0x02B9;
const VENDOR_USAGE_PAGE: u16 = 0xFF00;

const CMD_REPORT_ID: u8 = 0x05;
const CMD_REPORT_LEN: usize = 6;
const DATA_REPORT_ID: u8 = 0x04;
pub const DATA_REPORT_LEN: usize = 0x208;

const RETRIES: usize = 3;
const RETRY_DELAY: Duration = Duration::from_millis(200);
pub const STEP_DELAY: Duration = Duration::from_millis(40);

pub struct Mouse {
    dev: HidDevice,
}

impl Mouse {
    pub fn open() -> Result<Self> {
        let api = HidApi::new().context("não foi possível iniciar o HID")?;
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VENDOR_ID
                    && d.product_id() == PRODUCT_ID
                    && d.usage_page() == VENDOR_USAGE_PAGE
            })
            .ok_or_else(|| anyhow!("rato Trust GXT 970 não encontrado"))?;
        let dev = info.open_device(&api).map_err(|e| {
            anyhow!(
                "não foi possível abrir o rato ({e}). Se o erro for de permissões, autorize o terminal em Definições do Sistema > Privacidade e Segurança > Monitorização de entrada"
            )
        })?;
        Ok(Self { dev })
    }

    fn retry<T>(mut op: impl FnMut() -> hidapi::HidResult<T>, what: &str) -> Result<T> {
        let mut last = None;
        for _ in 0..RETRIES {
            match op() {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last = Some(e);
                    sleep(RETRY_DELAY);
                }
            }
        }
        Err(anyhow!("{what} falhou: {}", last.unwrap()))
    }

    pub fn send_cmd(&self, cmd: [u8; 5]) -> Result<()> {
        let mut buf = [0u8; CMD_REPORT_LEN];
        buf[0] = CMD_REPORT_ID;
        buf[1..].copy_from_slice(&cmd);
        Self::retry(|| self.dev.send_feature_report(&buf), "envio de comando")
    }

    pub fn get_cmd(&self) -> Result<[u8; 5]> {
        let mut buf = [0u8; CMD_REPORT_LEN];
        buf[0] = CMD_REPORT_ID;
        let n = Self::retry(|| self.dev.get_feature_report(&mut buf), "leitura de comando")?;
        if n < 2 {
            bail!("resposta de comando curta ({n} bytes)");
        }
        let mut out = [0u8; 5];
        out.copy_from_slice(&buf[1..]);
        Ok(out)
    }

    pub fn send_data(&self, payload: &[u8]) -> Result<()> {
        if payload.len() > DATA_REPORT_LEN - 1 {
            bail!("bloco de dados demasiado grande");
        }
        let mut buf = [0u8; DATA_REPORT_LEN];
        buf[0] = DATA_REPORT_ID;
        buf[1..1 + payload.len()].copy_from_slice(payload);
        Self::retry(|| self.dev.send_feature_report(&buf), "envio de dados")
    }

    pub fn get_data(&self) -> Result<[u8; DATA_REPORT_LEN]> {
        let mut buf = [0u8; DATA_REPORT_LEN];
        buf[0] = DATA_REPORT_ID;
        let n = Self::retry(|| self.dev.get_feature_report(&mut buf), "leitura de dados")?;
        if n < 16 {
            bail!("resposta de dados curta ({n} bytes)");
        }
        Ok(buf)
    }

    pub fn query(&self, cmd: [u8; 5]) -> Result<[u8; 5]> {
        sleep(STEP_DELAY);
        self.send_cmd(cmd)?;
        sleep(STEP_DELAY);
        self.get_cmd()
    }

    pub fn query_data(&self, cmd: [u8; 5]) -> Result<[u8; DATA_REPORT_LEN]> {
        sleep(STEP_DELAY);
        self.send_cmd(cmd)?;
        sleep(STEP_DELAY);
        self.get_data()
    }
}
