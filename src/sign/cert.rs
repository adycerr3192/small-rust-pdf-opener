//! Certificate-based PDF signing (PKCS#12 → PKCS#7 / CMS).

use std::fs;
use std::path::Path;

use lopdf::{Dictionary, Document, Object, ObjectId, Stream, StringFormat};
use openssl::pkcs12::Pkcs12;
use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::pkey::{PKey, Private};
use openssl::stack::Stack;
use openssl::x509::X509;

use crate::error::{AppError, Result};

pub struct CertIdentity {
    pub key: PKey<Private>,
    pub cert: X509,
    pub chain: Vec<X509>,
    pub subject: String,
}

impl CertIdentity {
    pub fn from_pkcs12(path: impl AsRef<Path>, password: &str) -> Result<Self> {
        let data = fs::read(path.as_ref())?;
        let pkcs12 = Pkcs12::from_der(&data)
            .map_err(|e| AppError::sign(format!("Invalid PKCS#12: {e}")))?;
        let parsed = pkcs12
            .parse2(password)
            .map_err(|e| AppError::sign(format!("PKCS#12 password/parse: {e}")))?;
        let key = parsed
            .pkey
            .ok_or_else(|| AppError::sign("PKCS#12 has no private key"))?;
        let cert = parsed
            .cert
            .ok_or_else(|| AppError::sign("PKCS#12 has no certificate"))?;
        let mut chain = Vec::new();
        if let Some(ca) = parsed.ca {
            for c in ca {
                chain.push(c);
            }
        }
        let subject = format!("{:?}", cert.subject_name());
        Ok(Self {
            key,
            cert,
            chain,
            subject,
        })
    }
}

const PLACEHOLDER_LEN: usize = 8192;

/// Embed a cryptographic signature into PDF bytes.
///
/// `page_index` is 0-based. `rect` is `[llx, lly, urx, ury]` in PDF user space.
pub fn sign_pdf_bytes(
    pdf_bytes: &[u8],
    identity: &CertIdentity,
    page_index: usize,
    rect: [f32; 4],
    reason: &str,
) -> Result<Vec<u8>> {
    let mut doc = Document::load_mem(pdf_bytes)
        .map_err(|e| AppError::sign(format!("lopdf load: {e}")))?;

    let pages = doc.get_pages();
    let page_ids: Vec<ObjectId> = pages.values().copied().collect();
    if page_index >= page_ids.len() {
        return Err(AppError::sign("Page out of range for signing"));
    }
    let page_id = page_ids[page_index];

    let placeholder = vec![b'0'; PLACEHOLDER_LEN * 2];

    let mut sig_dict = Dictionary::new();
    sig_dict.set("Type", Object::Name(b"Sig".to_vec()));
    sig_dict.set("Filter", Object::Name(b"Adobe.PPKLite".to_vec()));
    sig_dict.set("SubFilter", Object::Name(b"adbe.pkcs7.detached".to_vec()));
    sig_dict.set(
        "Contents",
        Object::String(placeholder, StringFormat::Hexadecimal),
    );
    // Wide ByteRange placeholder so we can patch in-place with spaces.
    sig_dict.set(
        "ByteRange",
        Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
        ]),
    );
    sig_dict.set(
        "Reason",
        Object::String(reason.as_bytes().to_vec(), StringFormat::Literal),
    );
    sig_dict.set(
        "M",
        Object::String(format_pdf_date().into_bytes(), StringFormat::Literal),
    );

    let sig_id = doc.add_object(Object::Dictionary(sig_dict));

    let ap_content = format!(
        "q\n{w:.2} 0 0 {h:.2} 0 0 cm\n0 0 0 rg\n0 0 {w:.2} {h:.2} re\nS\nQ\n",
        w = (rect[2] - rect[0]).abs(),
        h = (rect[3] - rect[1]).abs(),
    );
    let ap_stream = Stream::new(Dictionary::new(), ap_content.into_bytes());
    let ap_id = doc.add_object(Object::Stream(ap_stream));

    let mut ap_dict = Dictionary::new();
    ap_dict.set("N", Object::Reference(ap_id));

    let mut widget = Dictionary::new();
    widget.set("Type", Object::Name(b"Annot".to_vec()));
    widget.set("Subtype", Object::Name(b"Widget".to_vec()));
    widget.set("FT", Object::Name(b"Sig".to_vec()));
    widget.set(
        "Rect",
        Object::Array(vec![
            Object::Real(rect[0]),
            Object::Real(rect[1]),
            Object::Real(rect[2]),
            Object::Real(rect[3]),
        ]),
    );
    widget.set("V", Object::Reference(sig_id));
    widget.set(
        "T",
        Object::String(b"Signature1".to_vec(), StringFormat::Literal),
    );
    widget.set("F", Object::Integer(4));
    widget.set("P", Object::Reference(page_id));
    widget.set("AP", Object::Dictionary(ap_dict));

    let widget_id = doc.add_object(Object::Dictionary(widget));

    {
        let page = doc
            .get_object_mut(page_id)
            .map_err(|e| AppError::sign(format!("page: {e}")))?;
        let dict = page
            .as_dict_mut()
            .map_err(|e| AppError::sign(format!("page dict: {e}")))?;
        match dict.get_mut(b"Annots") {
            Ok(Object::Array(arr)) => arr.push(Object::Reference(widget_id)),
            _ => {
                dict.set("Annots", Object::Array(vec![Object::Reference(widget_id)]));
            }
        }
    }

    let root_id = doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| AppError::sign("Missing document Root"))?;

    {
        let has_acro = doc
            .get_object(root_id)
            .ok()
            .and_then(|c| c.as_dict().ok())
            .and_then(|d| d.get(b"AcroForm").ok())
            .and_then(|o| o.as_reference().ok());

        if let Some(af_ref) = has_acro {
            let af = doc
                .get_object_mut(af_ref)
                .map_err(|e| AppError::sign(format!("AcroForm: {e}")))?;
            let af_dict = af
                .as_dict_mut()
                .map_err(|e| AppError::sign(format!("AcroForm dict: {e}")))?;
            match af_dict.get_mut(b"Fields") {
                Ok(Object::Array(arr)) => arr.push(Object::Reference(widget_id)),
                _ => {
                    af_dict.set(
                        "Fields",
                        Object::Array(vec![Object::Reference(widget_id)]),
                    );
                }
            }
            af_dict.set("SigFlags", Object::Integer(3));
        } else {
            let mut af = Dictionary::new();
            af.set(
                "Fields",
                Object::Array(vec![Object::Reference(widget_id)]),
            );
            af.set("SigFlags", Object::Integer(3));
            let af_id = doc.add_object(Object::Dictionary(af));
            let catalog = doc
                .get_object_mut(root_id)
                .map_err(|e| AppError::sign(format!("catalog: {e}")))?;
            let dict = catalog
                .as_dict_mut()
                .map_err(|e| AppError::sign(format!("catalog dict: {e}")))?;
            dict.set("AcroForm", Object::Reference(af_id));
        }
    }

    let mut prepared = Vec::new();
    doc.save_to(&mut prepared)
        .map_err(|e| AppError::sign(format!("save: {e}")))?;

    // Widen /ByteRange[...] first so Contents offsets stay stable afterwards.
    expand_byte_range_slot(&mut prepared)?;

    let contents_marker = b"/Contents<";
    let start = prepared
        .windows(contents_marker.len())
        .position(|w| w == contents_marker)
        .ok_or_else(|| AppError::sign("Could not locate /Contents placeholder"))?
        + contents_marker.len();
    let end = start + PLACEHOLDER_LEN * 2;
    if end >= prepared.len() || prepared.get(end) != Some(&b'>') {
        return Err(AppError::sign("Contents placeholder layout mismatch"));
    }

    let byte_range = [
        0i64,
        start as i64,
        (end + 1) as i64,
        (prepared.len() - end - 1) as i64,
    ];
    write_byte_range(&mut prepared, &byte_range)?;

    let mut to_sign = Vec::with_capacity((byte_range[1] + byte_range[3]) as usize);
    to_sign.extend_from_slice(&prepared[0..byte_range[1] as usize]);
    to_sign.extend_from_slice(&prepared[byte_range[2] as usize..]);

    let pkcs7_der = create_pkcs7(identity, &to_sign)?;
    if pkcs7_der.len() > PLACEHOLDER_LEN {
        return Err(AppError::sign(format!(
            "Signature too large ({} > {}) — try a smaller certificate chain",
            pkcs7_der.len(),
            PLACEHOLDER_LEN
        )));
    }

    let mut hex = hex_encode(&pkcs7_der);
    while hex.len() < PLACEHOLDER_LEN * 2 {
        hex.push(b'0');
    }
    prepared[start..end].copy_from_slice(&hex);

    Ok(prepared)
}

const BYTE_RANGE_SLOT: usize = 64;

fn find_byte_range_brackets(prepared: &[u8]) -> Result<(usize, usize)> {
    let br_marker = b"/ByteRange";
    let br_pos = prepared
        .windows(br_marker.len())
        .position(|w| w == br_marker)
        .ok_or_else(|| AppError::sign("Could not locate /ByteRange"))?;
    let br_arr_start = prepared[br_pos..]
        .iter()
        .position(|&b| b == b'[')
        .ok_or_else(|| AppError::sign("ByteRange '[' missing"))?
        + br_pos;
    let br_arr_end = prepared[br_arr_start..]
        .iter()
        .position(|&b| b == b']')
        .ok_or_else(|| AppError::sign("ByteRange ']' missing"))?
        + br_arr_start;
    Ok((br_arr_start, br_arr_end))
}

fn expand_byte_range_slot(prepared: &mut Vec<u8>) -> Result<()> {
    let (start, end) = find_byte_range_brackets(prepared)?;
    let old_len = end - start + 1;
    if old_len >= BYTE_RANGE_SLOT {
        return Ok(());
    }
    let mut out = Vec::with_capacity(prepared.len() + BYTE_RANGE_SLOT);
    out.extend_from_slice(&prepared[..start]);
    out.push(b'[');
    out.extend(std::iter::repeat_n(b' ', BYTE_RANGE_SLOT - 2));
    out.push(b']');
    out.extend_from_slice(&prepared[end + 1..]);
    *prepared = out;
    Ok(())
}

fn write_byte_range(prepared: &mut [u8], byte_range: &[i64; 4]) -> Result<()> {
    let (start, end) = find_byte_range_brackets(prepared)?;
    let slot = end - start + 1;
    let br_str = format!(
        "[{} {} {} {}]",
        byte_range[0], byte_range[1], byte_range[2], byte_range[3]
    );
    if br_str.len() > slot {
        return Err(AppError::sign("ByteRange values exceed padded slot"));
    }
    let mut padded = br_str;
    while padded.len() < slot {
        let insert_at = padded.len() - 1;
        padded.insert(insert_at, ' ');
    }
    prepared[start..=end].copy_from_slice(padded.as_bytes());
    Ok(())
}

fn create_pkcs7(identity: &CertIdentity, data: &[u8]) -> Result<Vec<u8>> {
    let mut certs = Stack::new().map_err(|e| AppError::sign(e.to_string()))?;
    for c in &identity.chain {
        certs
            .push(c.clone())
            .map_err(|e| AppError::sign(e.to_string()))?;
    }
    let pkcs7 = Pkcs7::sign(
        &identity.cert,
        &identity.key,
        &certs,
        data,
        Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY,
    )
    .map_err(|e| AppError::sign(format!("PKCS7 sign: {e}")))?;
    pkcs7
        .to_der()
        .map_err(|e| AppError::sign(format!("PKCS7 DER: {e}")))
}

fn hex_encode(data: &[u8]) -> Vec<u8> {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut out = Vec::with_capacity(data.len() * 2);
    for &b in data {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0xf) as usize]);
    }
    out
}

fn format_pdf_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86400) as i64;
    let (y, m, d) = civil_from_days(days);
    let hh = (secs / 3600) % 24;
    let mm = (secs / 60) % 60;
    let ss = secs % 60;
    format!("D:{y:04}{m:02}{d:02}{hh:02}{mm:02}{ss:02}Z")
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
