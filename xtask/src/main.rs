//! Utilitários de build do akribie.
//!
//! `cargo run -p xtask -- font build` materializa `assets/Excalifont-Regular.ttf` a partir do
//! woff2 oficial, verificado contra o SHA-256 registrado em `assets/FONT.lock`.
//!
//! O binário é ferramenta de build, não domínio: pode falar com a rede (via `curl`) e com o
//! sistema de arquivos diretamente. Nada aqui atravessa `akribie-core`.

use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

/// URL oficial do Excalifont (OFL-1.1), citada no ADR-0001 §5.5.
const FONT_URL: &str =
    "https://excalidraw.nyc3.cdn.digitaloceanspaces.com/fonts/Excalifont-Regular.woff2";

/// Nome do arquivo no diretório `assets/`.
const FONT_FILE: &str = "Excalifont-Regular.woff2";

/// Constantes de SHA-256 (FIPS 180-4).
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["font", "build"] => font_build()?,
        [] => return usage(None),
        other => return usage(Some(&other.join(" "))),
    }
    Ok(())
}

fn usage(received: Option<&str>) -> Result<(), Box<dyn Error>> {
    match received {
        Some(cmd) => eprintln!("xtask: comando desconhecido `{cmd}`"),
        None => eprintln!("xtask: faltou o comando"),
    }
    eprintln!("uso: cargo run -p xtask -- font build");
    Err("comando inválido".into())
}

/// Materializa o `.ttf` a partir do woff2 pinado, sem regravar quando o resultado é o mesmo.
fn font_build() -> Result<(), Box<dyn Error>> {
    let root = repo_root();
    let assets = root.join("assets");
    let woff2_path = assets.join(FONT_FILE);
    let ttf_path = assets.join("Excalifont-Regular.ttf");
    let lock_path = assets.join("FONT.lock");

    if !woff2_path.exists() {
        eprintln!("xtask: baixando {FONT_URL}");
        download(FONT_URL, &woff2_path)?;
    }

    let woff2 = fs::read(&woff2_path)?;
    let digest = hex(&sha256(&woff2));

    let pinned = match fs::read_to_string(&lock_path) {
        Ok(text) => Some(parse_lock(&text)),
        Err(_) => None,
    };

    match &pinned {
        Some(expected) if expected == &digest => {}
        Some(expected) => {
            return Err(format!(
                "xtask: SHA-256 de {FONT_FILE} não corresponde ao pino.\n  esperado: {expected}\n  obtido:   {digest}\n  atualize assets/FONT.lock de forma consciente"
            )
            .into());
        }
        None => {
            let line = format!("{digest}  {FONT_FILE}\n");
            fs::write(&lock_path, line)?;
            eprintln!("xtask: pino gravado em assets/FONT.lock ({digest})");
        }
    }

    let ttf = woff2::decode::convert_woff2_to_ttf(&mut Cursor::new(&woff2))
        .map_err(|err| format!("xtask: falha ao converter woff2 → ttf: {err:?}"))?;

    match fs::read(&ttf_path) {
        Ok(existing) if existing == ttf => {
            eprintln!(
                "xtask: assets/Excalifont-Regular.ttf inalterado ({} bytes)",
                ttf.len()
            );
        }
        _ => {
            fs::write(&ttf_path, &ttf)?;
            eprintln!(
                "xtask: assets/Excalifont-Regular.ttf gravado ({} bytes)",
                ttf.len()
            );
        }
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn download(url: &str, dest: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let status = Command::new("curl")
        .args(["-fsSL", "--max-time", "120", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|err| {
            format!("xtask: `curl` indisponível (necessário para baixar a fonte): {err}")
        })?;
    if !status.success() {
        return Err(format!("xtask: download falhou (curl saiu com {status})").into());
    }
    Ok(())
}

/// Primeira palavra do `FONT.lock` (formato `<hex>  <arquivo>`).
fn parse_lock(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// SHA-256 (FIPS 180-4). Implementado aqui para não acrescentar dependência ao build.
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut message = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let mut bytes = [0u8; 4];
            for (j, slot) in bytes.iter_mut().enumerate() {
                *slot = chunk.get(i * 4 + j).copied().unwrap_or(0);
            }
            *word = u32::from_be_bytes(bytes);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        state = [
            state[0].wrapping_add(a),
            state[1].wrapping_add(b),
            state[2].wrapping_add(c),
            state[3].wrapping_add(d),
            state[4].wrapping_add(e),
            state[5].wrapping_add(f),
            state[6].wrapping_add(g),
            state[7].wrapping_add(h),
        ];
    }

    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}
