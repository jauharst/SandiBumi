//! DLIS (RP66 V1) writer — SB-CORE-015 under DEC-054.
//!
//! Every structure below is built from the normative API RP66 V1 specification at
//! https://energistics.org/sites/default/files/RP66/V1/Toc/main.html and cited by section
//! number; per DEC-054 constraint 1 no specification text is copied into this repository —
//! the URL plus a section reference IS the citation. The writer is native Rust on purpose:
//! `dlisio` (the import route, rule 7 subprocess) has no write capability at all, so a
//! subprocess writer does not exist to delegate to (DEC-054 constraint 3 left the route an
//! engineering question; this is the answer and the reason).
//!
//! Layout written (one Storage Unit, one Logical File):
//!   Storage Unit Label (§2.3.2) → Visible Records (§2.3.6) each carrying Logical Record
//!   Segments (§2.2.2) that reassemble into: FILE-HEADER EFLR (type 0, §5.1, Appendix A
//!   Figure A-2) → ORIGIN EFLR (type 1, §5.2) → CHANNEL EFLR (type 3, §5.5) → FRAME EFLR
//!   (type 4, §5.7) → one FDATA IFLR (type 0, Appendix A Figure A-1, §5.6) per depth frame.
//!
//! Two deliberate, conforming choices:
//! - No Logical Record Segment carries an encryption packet or a checksum: §2.2.2.1
//!   Figure 2-3 defines both as attribute bits an LRS may leave clear, and the Appendix E
//!   checksum is therefore not emitted. The round-trip self-check is the integrity gate.
//! - Missing samples are written as IEEE 754 NaN in FSINGL (Appendix B code 2 is IEEE
//!   single precision) — DLIS has no null-sentinel convention to honour, rule 2's NaN is
//!   representable natively, and `dlisio` returns it back as NaN, which is exactly what the
//!   generic store holds. The export result therefore reports zero precision reduction:
//!   FSINGL is the stored f32, bit for bit.
//!
//! Component Descriptor bytes (§3.2.2.1 Figures 3-2..3-5): the role sits in bits 1-3
//! (Figure 3-2: SET=111, OBJECT=011, ATTRIB=001) and the format flags follow in bits 4-8.
//! The HTML rendering of Figures 3-3/3-4/3-5 carries the flag table graphically, so the
//! flag assignment used here (Set: Type=0x10, Name=0x08; Object: Name=0x10; Attribute:
//! Label=0x10, Count=0x08, RepCode=0x04, Units=0x02, Value=0x01) is additionally pinned
//! empirically by the dlisio round-trip test — a wrong bit does not parse.

/// Appendix B representation codes used by this writer.
pub const RC_FSINGL: u8 = 2; // IEEE 754 single precision, big-endian (Appendix B code 2)
pub const RC_USHORT: u8 = 15;
pub const RC_UVARI: u8 = 18;
pub const RC_IDENT: u8 = 19;
pub const RC_ASCII: u8 = 20;
pub const RC_OBNAME: u8 = 23;
pub const RC_UNITS: u8 = 27;

/// §2.3.6.4: a Visible Record is at most 16,384 bytes; the SUL's Maximum Record Length
/// field declares what THIS storage unit uses. 8 192 keeps every record well inside the cap.
pub const MAX_VISIBLE_RECORD: usize = 8192;
/// One Visible Record spends 4 bytes on its own envelope: UNORM length + 0xFF 0x01
/// format-version pair (§2.3.6.2).
const VR_HEADER: usize = 4;
/// §2.2.2.1 comment 1: a Logical Record Segment is at least sixteen bytes.
const MIN_SEGMENT: usize = 16;

// --- Appendix B scalar encodings -------------------------------------------------------

/// UVARI (Appendix B code 18): 1 byte below 128, 2 bytes (high bit 10) to 16 383,
/// 4 bytes (high bits 11) to 2^30 - 1.
pub fn uvari(value: u32, out: &mut Vec<u8>) {
    if value < 0x80 {
        out.push(value as u8);
    } else if value < 0x4000 {
        out.extend_from_slice(&(0x8000u16 | value as u16).to_be_bytes());
    } else {
        assert!(value < 0x4000_0000, "UVARI value out of the 30-bit range");
        out.extend_from_slice(&(0xC000_0000u32 | value).to_be_bytes());
    }
}

/// IDENT (Appendix B code 19): USHORT length then ASCII characters. Non-ASCII bytes are
/// replaced with '?' rather than emitting a byte the 7-bit form cannot carry.
pub fn ident(text: &str, out: &mut Vec<u8>) {
    let bytes: Vec<u8> =
        text.bytes().take(255).map(|b| if b.is_ascii() && b >= 0x20 { b } else { b'?' }).collect();
    out.push(bytes.len() as u8);
    out.extend_from_slice(&bytes);
}

/// ASCII (Appendix B code 20): UVARI length then characters.
pub fn ascii_v(text: &str, out: &mut Vec<u8>) {
    let bytes: Vec<u8> = text.bytes().map(|b| if b.is_ascii() { b } else { b'?' }).collect();
    uvari(bytes.len() as u32, out);
    out.extend_from_slice(&bytes);
}

/// OBNAME (Appendix B code 23): Origin reference (UVARI form) + Copy Number (USHORT) +
/// Identifier (IDENT).
pub fn obname(origin: u32, copy: u8, id: &str, out: &mut Vec<u8>) {
    uvari(origin, out);
    out.push(copy);
    ident(id, out);
}

pub fn fsingl(value: f32, out: &mut Vec<u8>) {
    out.extend_from_slice(&value.to_be_bytes());
}

// --- §3.2.2 EFLR component builders ----------------------------------------------------

/// §3.2.2.1 Figure 3-2 roles in bits 1-3 (high order), format flags in bits 4-8.
const ROLE_ATTRIB: u8 = 0b001_00000;
const ROLE_OBJECT: u8 = 0b011_00000;
const ROLE_SET: u8 = 0b111_00000;
const SET_HAS_TYPE: u8 = 0x10;
const OBJECT_HAS_NAME: u8 = 0x10;
const ATTR_LABEL: u8 = 0x10;
const ATTR_COUNT: u8 = 0x08;
const ATTR_RC: u8 = 0x04;
const ATTR_UNITS: u8 = 0x02;
const ATTR_VALUE: u8 = 0x01;

/// A Set Component with an explicit Type and no Name (§3.2.2.2: the Type is mandatory,
/// the Name optional).
fn set_component(set_type: &str, out: &mut Vec<u8>) {
    out.push(ROLE_SET | SET_HAS_TYPE);
    ident(set_type, out);
}

/// A Template attribute declaring its Label and Representation Code; Count keeps the
/// global default of 1 (§3.2.2.1 Figure 3-5).
fn template_attr(label: &str, rc: u8, out: &mut Vec<u8>) {
    out.push(ROLE_ATTRIB | ATTR_LABEL | ATTR_RC);
    ident(label, out);
    out.push(rc);
}

fn object_component(origin: u32, id: &str, out: &mut Vec<u8>) {
    out.push(ROLE_OBJECT | OBJECT_HAS_NAME);
    obname(origin, 0, id, out);
}

/// An object Attribute carrying only its Value; count and representation come from the
/// template's local defaults (§3.2.2.2).
fn value_attr(encode: impl FnOnce(&mut Vec<u8>), out: &mut Vec<u8>) {
    out.push(ROLE_ATTRIB | ATTR_VALUE);
    encode(out);
}

/// An object Attribute carrying Units and Value.
fn units_value_attr(units: &str, encode: impl FnOnce(&mut Vec<u8>), out: &mut Vec<u8>) {
    out.push(ROLE_ATTRIB | ATTR_UNITS | ATTR_VALUE);
    ident(units, out); // UNITS (code 27) shares IDENT's single-byte-length form
    encode(out);
}

/// An object Attribute carrying an explicit Count and Value (a list).
fn count_value_attr(count: u32, encode: impl FnOnce(&mut Vec<u8>), out: &mut Vec<u8>) {
    out.push(ROLE_ATTRIB | ATTR_COUNT | ATTR_VALUE);
    uvari(count, out);
    encode(out);
}

// --- Logical record assembly (§2.2.2, §2.3.6) ------------------------------------------

/// One fully assembled logical record body plus its framing type.
#[derive(Debug)]
pub struct LogicalRecord {
    pub lr_type: u8,
    pub is_eflr: bool,
    pub body: Vec<u8>,
}

/// Splits logical records into Logical Record Segments and packs them into Visible
/// Records: §2.2.2.1 header (UNORM even length, attribute byte, USHORT type), predecessor/
/// successor bits across splits, pad bytes with a trailing pad count when the body is odd
/// or short (§2.2.2.4: the Pad Count is one of the Pad Bytes), §2.3.6 Visible Record
/// envelope of UNORM length + 0xFF 0x01.
pub fn assemble(records: &[LogicalRecord]) -> Vec<u8> {
    let max_seg_body = MAX_VISIBLE_RECORD - VR_HEADER - 4; // one segment must fit one VR
    let mut segments: Vec<Vec<u8>> = Vec::new();
    for record in records {
        let chunks: Vec<&[u8]> = if record.body.is_empty() {
            vec![&record.body[..]]
        } else {
            record.body.chunks(max_seg_body).collect()
        };
        let last = chunks.len() - 1;
        for (index, chunk) in chunks.iter().enumerate() {
            let mut attrs: u8 = 0;
            if record.is_eflr {
                attrs |= 0x80; // §2.2.2.1 Figure 2-3 bit 1: explicitly formatted
            }
            if index > 0 {
                attrs |= 0x40; // bit 2: has predecessor
            }
            if index < last {
                attrs |= 0x20; // bit 3: has successor
            }
            // Pad to even length and to the 16-byte minimum. The pad count byte is itself
            // one of the pad bytes (§2.2.2.4), so padding is always at least one byte when
            // the padding bit is set.
            let unpadded = 4 + chunk.len();
            let mut pad = if unpadded % 2 == 1 { 1 } else { 0 };
            while unpadded + pad < MIN_SEGMENT {
                pad += 2;
            }
            if pad > 0 {
                attrs |= 0x01; // bit 8: padding present
            }
            let total = unpadded + pad;
            let mut seg = Vec::with_capacity(total);
            seg.extend_from_slice(&(total as u16).to_be_bytes());
            seg.push(attrs);
            seg.push(record.lr_type);
            seg.extend_from_slice(chunk);
            if pad > 0 {
                seg.extend(std::iter::repeat(0u8).take(pad - 1));
                seg.push(pad as u8); // pad count, last byte of the trailer
            }
            segments.push(seg);
        }
    }

    let mut out = Vec::new();
    let mut vr: Vec<u8> = Vec::new();
    let flush = |vr: &mut Vec<u8>, out: &mut Vec<u8>| {
        if vr.is_empty() {
            return;
        }
        let total = (vr.len() + VR_HEADER) as u16;
        out.extend_from_slice(&total.to_be_bytes());
        out.push(0xFF); // §2.3.6.2 format version byte 1
        out.push(0x01); // §2.3.6.2 major version (USHORT 1)
        out.append(vr);
    };
    for seg in segments {
        if VR_HEADER + vr.len() + seg.len() > MAX_VISIBLE_RECORD {
            flush(&mut vr, &mut out);
        }
        vr.extend_from_slice(&seg);
    }
    flush(&mut vr, &mut out);
    out
}

/// §2.3.2 Storage Unit Label: 80 ASCII bytes — sequence number (4, right justified),
/// DLIS version (5, "V1.00"), storage unit structure (6, "RECORD", left justified),
/// maximum record length (5, right justified), storage set identifier (60).
pub fn storage_unit_label(storage_set_id: &str) -> [u8; 80] {
    let mut label = [b' '; 80];
    label[0..4].copy_from_slice(b"   1");
    label[4..9].copy_from_slice(b"V1.00");
    label[9..15].copy_from_slice(b"RECORD");
    label[15..20].copy_from_slice(b" 8192");
    let id: Vec<u8> =
        storage_set_id.bytes().take(60).map(|b| if b.is_ascii() && b >= 0x20 { b } else { b'?' }).collect();
    label[20..20 + id.len()].copy_from_slice(&id);
    label
}

// --- The file --------------------------------------------------------------------------

/// One channel to write: mnemonic, unit string, and its samples aligned to the depth frame.
pub struct DlisChannel {
    pub mnemonic: String,
    pub unit: String,
    pub description: String,
}

/// Everything the byte assembly needs; gathering it from the project is the caller's job
/// (`export.rs`), so this half stays a pure, testable function of its inputs.
pub struct DlisFileSpec {
    pub well_name: String,
    pub field_name: String,
    pub depth_unit: String,
    pub depth: Vec<f32>,
    /// Channels beside the index; `columns[i]` aligns with `channels[i]` and must match
    /// `depth.len()`.
    pub channels: Vec<DlisChannel>,
    pub columns: Vec<Vec<f32>>,
}

/// The one origin/copy identity used for every OBNAME in the file; the ORIGIN record's
/// FILE-SET-NUMBER and FILE-NUMBER carry the same value so the references resolve.
const ORIGIN_REF: u32 = 1;
const FRAME_NAME: &str = "FRAME_STANDARD";
const INDEX_MNEMONIC: &str = "DEPT";

pub fn build_logical_records(spec: &DlisFileSpec) -> Result<Vec<LogicalRecord>, String> {
    if spec.depth.is_empty() {
        return Err("well has no curve data".into());
    }
    for (index, column) in spec.columns.iter().enumerate() {
        if column.len() != spec.depth.len() {
            return Err(format!(
                "channel {} has {} samples but the depth frame has {}",
                spec.channels.get(index).map(|c| c.mnemonic.as_str()).unwrap_or("?"),
                column.len(),
                spec.depth.len()
            ));
        }
    }

    let mut records = Vec::new();

    // FILE-HEADER (§5.1): first logical record; SEQUENCE-NUMBER is 10 ASCII characters
    // right justified, ID is 65 characters.
    let mut body = Vec::new();
    set_component("FILE-HEADER", &mut body);
    template_attr("SEQUENCE-NUMBER", RC_ASCII, &mut body);
    template_attr("ID", RC_ASCII, &mut body);
    object_component(ORIGIN_REF, "0", &mut body);
    value_attr(|b| ascii_v(&format!("{:>10}", 1), b), &mut body);
    let id65 = format!("{:<65.65}", format!("SandiBumi export: {}", spec.well_name));
    value_attr(|b| ascii_v(&id65, b), &mut body);
    records.push(LogicalRecord { lr_type: 0, is_eflr: true, body });

    // ORIGIN (§5.2): the defining origin of the logical file.
    let mut body = Vec::new();
    set_component("ORIGIN", &mut body);
    template_attr("FILE-ID", RC_ASCII, &mut body);
    template_attr("FILE-SET-NAME", RC_IDENT, &mut body);
    template_attr("FILE-SET-NUMBER", RC_UVARI, &mut body);
    template_attr("FILE-NUMBER", RC_UVARI, &mut body);
    template_attr("FILE-TYPE", RC_IDENT, &mut body);
    template_attr("WELL-NAME", RC_ASCII, &mut body);
    template_attr("FIELD-NAME", RC_ASCII, &mut body);
    template_attr("COMPANY", RC_ASCII, &mut body);
    object_component(ORIGIN_REF, "SANDIBUMI", &mut body);
    value_attr(|b| ascii_v(&format!("SandiBumi export: {}", spec.well_name), b), &mut body);
    value_attr(|b| ident("SANDIBUMI", b), &mut body);
    value_attr(|b| uvari(ORIGIN_REF, b), &mut body);
    value_attr(|b| uvari(ORIGIN_REF, b), &mut body);
    value_attr(|b| ident("PLAYBACK", b), &mut body);
    value_attr(|b| ascii_v(&spec.well_name, b), &mut body);
    value_attr(|b| ascii_v(&spec.field_name, b), &mut body);
    value_attr(|b| ascii_v("SandiBumi", b), &mut body);
    records.push(LogicalRecord { lr_type: 1, is_eflr: true, body });

    // CHANNEL (§5.5): the index channel first, then every exported curve, all FSINGL.
    let mut body = Vec::new();
    set_component("CHANNEL", &mut body);
    template_attr("LONG-NAME", RC_ASCII, &mut body);
    template_attr("REPRESENTATION-CODE", RC_USHORT, &mut body);
    template_attr("UNITS", RC_UNITS, &mut body);
    template_attr("DIMENSION", RC_UVARI, &mut body);
    template_attr("ELEMENT-LIMIT", RC_UVARI, &mut body);
    let write_channel = |mnemonic: &str, unit: &str, long_name: &str, body: &mut Vec<u8>| {
        object_component(ORIGIN_REF, mnemonic, body);
        value_attr(|b| ascii_v(long_name, b), body);
        value_attr(|b| b.push(RC_FSINGL), body);
        value_attr(|b| ident(unit, b), body);
        value_attr(|b| uvari(1, b), body);
        value_attr(|b| uvari(1, b), body);
    };
    write_channel(INDEX_MNEMONIC, &spec.depth_unit, "Measured depth index", &mut body);
    for channel in &spec.channels {
        write_channel(&channel.mnemonic, &channel.unit, &channel.description, &mut body);
    }
    records.push(LogicalRecord { lr_type: 3, is_eflr: true, body });

    // FRAME (§5.7): one frame type over the standard depth grid, indexed by the first
    // channel (BOREHOLE-DEPTH, INCREASING). SPACING is deliberately absent: the standard
    // frame is not guaranteed uniform, and declaring a constant spacing over an irregular
    // index would let a conforming reader rebuild depths that were never measured — the
    // same reasoning as the LAS writer's STEP 0 declaration (SB-DIO-056).
    let mut body = Vec::new();
    set_component("FRAME", &mut body);
    template_attr("DESCRIPTION", RC_ASCII, &mut body);
    template_attr("CHANNELS", RC_OBNAME, &mut body);
    template_attr("INDEX-TYPE", RC_IDENT, &mut body);
    template_attr("DIRECTION", RC_IDENT, &mut body);
    template_attr("INDEX-MIN", RC_FSINGL, &mut body);
    template_attr("INDEX-MAX", RC_FSINGL, &mut body);
    object_component(ORIGIN_REF, FRAME_NAME, &mut body);
    value_attr(|b| ascii_v("SandiBumi standard depth frame", b), &mut body);
    let channel_count = (spec.channels.len() + 1) as u32;
    count_value_attr(
        channel_count,
        |b| {
            obname(ORIGIN_REF, 0, INDEX_MNEMONIC, b);
            for channel in &spec.channels {
                obname(ORIGIN_REF, 0, &channel.mnemonic, b);
            }
        },
        &mut body,
    );
    value_attr(|b| ident("BOREHOLE-DEPTH", b), &mut body);
    value_attr(|b| ident("INCREASING", b), &mut body);
    let first = *spec.depth.first().unwrap();
    let last = *spec.depth.last().unwrap();
    units_value_attr(&spec.depth_unit, |b| fsingl(first, b), &mut body);
    units_value_attr(&spec.depth_unit, |b| fsingl(last, b), &mut body);
    records.push(LogicalRecord { lr_type: 4, is_eflr: true, body });

    // FDATA IFLRs (Appendix A Figure A-1 type 0; §5.6): one frame per record — the Data
    // Descriptor Reference names the Frame object, then the 1-based frame number (UVARI),
    // then one FSINGL slot per channel with the index first.
    for (row, &depth) in spec.depth.iter().enumerate() {
        let mut body = Vec::new();
        obname(ORIGIN_REF, 0, FRAME_NAME, &mut body);
        uvari((row + 1) as u32, &mut body);
        fsingl(depth, &mut body);
        for column in &spec.columns {
            fsingl(column[row], &mut body);
        }
        records.push(LogicalRecord { lr_type: 0, is_eflr: false, body });
    }

    Ok(records)
}

/// Assembles the complete storage unit: SUL + packed visible records.
pub fn write_storage_unit(spec: &DlisFileSpec) -> Result<Vec<u8>, String> {
    let records = build_logical_records(spec)?;
    let mut out = Vec::new();
    out.extend_from_slice(&storage_unit_label(&format!("SANDIBUMI: {}", spec.well_name)));
    out.extend_from_slice(&assemble(&records));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_spec() -> DlisFileSpec {
        DlisFileSpec {
            well_name: "SANDI-01".into(),
            field_name: "SANDI".into(),
            depth_unit: "m".into(),
            depth: vec![1000.0, 1000.5, 1001.0],
            channels: vec![DlisChannel {
                mnemonic: "GR".into(),
                unit: "gAPI".into(),
                description: "Gamma ray".into(),
            }],
            columns: vec![vec![45.0, f32::NAN, 60.25]],
        }
    }

    #[test]
    fn the_storage_unit_label_is_the_80_ascii_bytes_section_2_3_2_defines() {
        let label = storage_unit_label("SANDIBUMI: SANDI-01");
        assert_eq!(label.len(), 80);
        assert_eq!(&label[0..4], b"   1", "sequence number, 4 chars right justified");
        assert_eq!(&label[4..9], b"V1.00", "DLIS version");
        assert_eq!(&label[9..15], b"RECORD", "storage unit structure");
        assert_eq!(&label[15..20], b" 8192", "maximum record length, right justified");
        assert!(label[20..].starts_with(b"SANDIBUMI: SANDI-01"));
        assert!(label.iter().all(u8::is_ascii), "the SUL is ASCII throughout");
    }

    #[test]
    fn uvari_ident_and_obname_encode_the_appendix_b_forms_across_their_length_boundaries() {
        let enc = |v: u32| {
            let mut b = Vec::new();
            uvari(v, &mut b);
            b
        };
        assert_eq!(enc(0), vec![0x00]);
        assert_eq!(enc(127), vec![0x7F], "127 is the last 1-byte value");
        assert_eq!(enc(128), vec![0x80, 0x80], "128 opens the 2-byte form");
        assert_eq!(enc(16383), vec![0xBF, 0xFF], "16383 is the last 2-byte value");
        assert_eq!(enc(16384), vec![0xC0, 0x00, 0x40, 0x00], "16384 opens the 4-byte form");

        let mut id = Vec::new();
        ident("GR", &mut id);
        assert_eq!(id, vec![2, b'G', b'R'], "IDENT is a USHORT length then characters");

        let mut ob = Vec::new();
        obname(1, 0, "DEPT", &mut ob);
        assert_eq!(
            ob,
            vec![1, 0, 4, b'D', b'E', b'P', b'T'],
            "OBNAME is origin UVARI + copy USHORT + IDENT"
        );

        let mut non_ascii = Vec::new();
        ident("R\u{00E9}S", &mut non_ascii);
        assert_eq!(non_ascii[0] as usize, non_ascii.len() - 1);
        assert!(non_ascii[1..].iter().all(u8::is_ascii), "IDENT never carries a non-ASCII byte");
    }

    /// Walks the assembled bytes with an independent reader: every Visible Record carries
    /// the 0xFF 0x01 envelope and its declared length; every Logical Record Segment is
    /// even, at least 16 bytes, and the predecessor/successor bits reassemble the exact
    /// pre-segmentation logical record bodies.
    #[test]
    fn segments_are_even_at_least_sixteen_bytes_and_reassemble_into_the_records_that_went_in() {
        let mut records = Vec::new();
        records.push(LogicalRecord { lr_type: 0, is_eflr: true, body: vec![0xAB; 5] });
        records.push(LogicalRecord { lr_type: 3, is_eflr: true, body: vec![0xCD; 9000] });
        records.push(LogicalRecord { lr_type: 0, is_eflr: false, body: vec![0xEF; 20] });
        let bytes = assemble(&records);

        let mut segments: Vec<(u8, u8, Vec<u8>)> = Vec::new();
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            let vr_len = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            assert!(vr_len <= MAX_VISIBLE_RECORD, "visible record within the declared maximum");
            assert_eq!(bytes[cursor + 2], 0xFF, "format version byte 1");
            assert_eq!(bytes[cursor + 3], 0x01, "format version byte 2");
            let vr_end = cursor + vr_len;
            let mut seg_at = cursor + 4;
            while seg_at < vr_end {
                let seg_len =
                    u16::from_be_bytes([bytes[seg_at], bytes[seg_at + 1]]) as usize;
                assert_eq!(seg_len % 2, 0, "segment length is even");
                assert!(seg_len >= MIN_SEGMENT, "segment carries at least sixteen bytes");
                let attrs = bytes[seg_at + 2];
                let lr_type = bytes[seg_at + 3];
                let mut body_end = seg_at + seg_len;
                if attrs & 0x01 != 0 {
                    let pad = bytes[body_end - 1] as usize;
                    assert!(pad >= 1 && pad < seg_len, "pad count within the segment");
                    body_end -= pad;
                }
                segments.push((attrs, lr_type, bytes[seg_at + 4..body_end].to_vec()));
                seg_at += seg_len;
            }
            assert_eq!(seg_at, vr_end, "segments fill the visible record exactly");
            cursor = vr_end;
        }

        let mut rebuilt: Vec<(u8, bool, Vec<u8>)> = Vec::new();
        for (attrs, lr_type, body) in segments {
            let has_predecessor = attrs & 0x40 != 0;
            if has_predecessor {
                rebuilt.last_mut().expect("predecessor bit implies an open record").2.extend(body);
            } else {
                rebuilt.push((lr_type, attrs & 0x80 != 0, body));
            }
        }
        assert_eq!(rebuilt.len(), records.len());
        for (got, want) in rebuilt.iter().zip(records.iter()) {
            assert_eq!(got.0, want.lr_type);
            assert_eq!(got.1, want.is_eflr);
            assert_eq!(got.2, want.body, "reassembled body is byte-identical");
        }
    }

    #[test]
    fn a_frame_row_writes_the_index_first_then_every_channel_as_fsingl_and_nan_stays_nan() {
        let spec = tiny_spec();
        let records = build_logical_records(&spec).unwrap();
        // 4 EFLRs then one FDATA per depth row.
        assert_eq!(records.len(), 4 + spec.depth.len());
        let fdata = &records[4 + 1]; // second row carries the NaN sample
        assert_eq!(fdata.lr_type, 0);
        assert!(!fdata.is_eflr);
        let body = &fdata.body;
        // OBNAME(1,0,FRAME_STANDARD) = 1 + 1 + 1 + len bytes, then UVARI frame number.
        let name_len = FRAME_NAME.len();
        assert_eq!(&body[0..3], &[1, 0, name_len as u8]);
        let mut at = 3 + name_len;
        assert_eq!(body[at], 2, "frame numbers are 1-based; the second row is frame 2");
        at += 1;
        let depth = f32::from_be_bytes(body[at..at + 4].try_into().unwrap());
        assert_eq!(depth, 1000.5, "the index slot comes first");
        at += 4;
        let gr = f32::from_be_bytes(body[at..at + 4].try_into().unwrap());
        assert!(gr.is_nan(), "a missing sample is IEEE NaN in FSINGL, not a sentinel");
        assert_eq!(body.len(), at + 4, "exactly one slot per channel");
    }

    #[test]
    fn a_channel_whose_column_length_disagrees_with_the_frame_is_refused_by_name() {
        let mut spec = tiny_spec();
        spec.columns[0].pop();
        let error = build_logical_records(&spec).unwrap_err();
        assert!(error.contains("GR"), "the refusal names the channel: {error}");
        assert!(error.contains('3') && error.contains('2'), "and both lengths: {error}");
    }
}
