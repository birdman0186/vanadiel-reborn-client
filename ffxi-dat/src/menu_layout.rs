// .agents/skills/retail-observe/references/original-ui-resources-2026-11-15.md
use crate::ui_element::{normalize_name, read_i16, read_u16, NAME_LEN};
use crate::{walk, ChunkKind};

const HEADER_LEN: usize = 32;
const CHILD_COUNT: usize = 17;
const RECORD_ALIGNMENT: usize = 16;
const RECORD_HEADER_LEN: usize = 32;
const ROOT_REF_COUNT: usize = 20;
const CHILD_REF_COUNT: usize = 27;
const X: usize = 2;
const Y: usize = 4;
const WIDTH: usize = 10;
const HEIGHT: usize = 12;
const OFFSET_X: usize = 14;
const OFFSET_Y: usize = 16;
const REF_INDEX: usize = 2;
const REF_NAME: usize = 4;
const REF_LEN: usize = REF_NAME + NAME_LEN;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuLayout {
    pub root: MenuRecord,
    pub children: Vec<MenuRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRecord {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub resources: Vec<MenuResource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuResource {
    pub tag: u16,
    pub index: u16,
    pub group: String,
}

pub fn find_menu_layout(bytes: &[u8], name: &str) -> Option<MenuLayout> {
    let name = normalize_name(name.as_bytes());
    walk(bytes)
        .flatten()
        .filter(|chunk| chunk.kind == ChunkKind::UiMenu as u8)
        .find_map(|chunk| {
            if normalize_name(chunk.data.get(..NAME_LEN)?) == name {
                parse_layout(chunk.data)
            } else {
                None
            }
        })
}

fn parse_layout(data: &[u8]) -> Option<MenuLayout> {
    let child_count = read_u16(data, CHILD_COUNT)?;
    let mut remaining = data.get(HEADER_LEN..)?;
    let root = take_record(&mut remaining, ROOT_REF_COUNT)?;
    let mut children = Vec::new();
    for _ in 0..child_count {
        children.push(take_record(&mut remaining, CHILD_REF_COUNT)?);
    }
    remaining
        .is_empty()
        .then_some(MenuLayout { root, children })
}

fn take_record(remaining: &mut &[u8], count_field: usize) -> Option<MenuRecord> {
    let len = usize::from(read_u16(remaining, 0)?);
    if !len.is_multiple_of(RECORD_ALIGNMENT) || len < RECORD_HEADER_LEN {
        return None;
    }
    let (record, rest) = remaining.split_at_checked(len)?;
    let count = usize::from(record[count_field]);
    let refs = record.get(RECORD_HEADER_LEN..RECORD_HEADER_LEN + count * REF_LEN)?;
    let mut resources = Vec::with_capacity(count);
    for entry in refs.chunks_exact(REF_LEN) {
        resources.push(MenuResource {
            tag: read_u16(entry, 0)?,
            index: read_u16(entry, REF_INDEX)?,
            group: normalize_name(&entry[REF_NAME..]),
        });
    }
    *remaining = rest;
    Some(MenuRecord {
        x: read_i16(record, X)?,
        y: read_i16(record, Y)?,
        width: read_u16(record, WIDTH)?,
        height: read_u16(record, HEIGHT)?,
        offset_x: read_i16(record, OFFSET_X)?,
        offset_y: read_i16(record, OFFSET_Y)?,
        resources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        let mut body = Vec::from(*b"test    panel   ");
        body.extend([1, 1]);
        body.resize(32, 0);
        for word in [80_i16, -41, 23, 0, 0, 95, 62, -3, 9] {
            body.extend(word.to_le_bytes());
        }
        body.extend([0, 1, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        body.extend([0, 0, 39, 0]);
        body.extend(b"test    panelbg ");
        body.extend([6, 0, 17, 1]);
        body.extend(b"test    pointer ");
        body.extend(b"-1\0-1\0\0\0");
        for word in [64_i16, 4, -2, 0, 0, 31, 25, 8, -7] {
            body.extend(word.to_le_bytes());
        }
        body.extend([1, 0, 0, 7, 3, 7, 3, 1, 1, 1, 0, 3, 2, 0]);
        body.extend([4, 0, 9, 1]);
        body.extend(b"test    button  ");
        body.extend(b"321\0abc\0\0\0\0\0");
        body
    }

    fn chunk(body: &[u8], kind: ChunkKind) -> Vec<u8> {
        let mut bytes = vec![0; 16];
        let size = (bytes.len() + body.len()).div_ceil(16);
        bytes[4..8].copy_from_slice(&((size as u32) << 7 | kind as u32).to_le_bytes());
        bytes.extend(body);
        bytes.resize(size * 16, 0);
        bytes
    }

    #[test]
    fn finds_the_named_layout_and_reads_each_record_in_file_order() {
        let body = fixture();
        let mut file = chunk(&body, ChunkKind::UiElementGroup);
        assert_eq!(find_menu_layout(&file, "test    panel"), None);
        file.extend(chunk(&body, ChunkKind::UiMenu));
        assert_eq!(find_menu_layout(&file, "test    missing"), None);
        let layout = find_menu_layout(&file, "TEST    PANEL   ").unwrap();
        let resource = |tag, index, group: &str| MenuResource {
            tag,
            index,
            group: group.into(),
        };
        assert_eq!(
            layout.root,
            MenuRecord {
                x: -41,
                y: 23,
                width: 95,
                height: 62,
                offset_x: -3,
                offset_y: 9,
                resources: vec![
                    resource(0, 39, "test    panelbg"),
                    resource(6, 273, "test    pointer"),
                ],
            }
        );
        assert_eq!(
            layout.children,
            [MenuRecord {
                x: 4,
                y: -2,
                width: 31,
                height: 25,
                offset_x: 8,
                offset_y: -7,
                resources: vec![resource(4, 265, "test    button")],
            }]
        );
    }

    #[test]
    fn rejects_missing_children_invalid_lengths_and_out_of_record_references() {
        let body = fixture();
        let root_start = HEADER_LEN;
        let child_start = root_start + 80;
        for (offset, value) in [
            (CHILD_COUNT, 0),
            (CHILD_COUNT, 2),
            (root_start, 0),
            (root_start, 16),
            (root_start, 79),
            (root_start, 255),
            (root_start + ROOT_REF_COUNT, 3),
            (child_start + CHILD_REF_COUNT, 2),
        ] {
            let mut invalid = body.clone();
            invalid[offset] = value;
            assert_eq!(
                parse_layout(&invalid),
                None,
                "offset {offset}, value {value}"
            );
        }
        for end in 0..body.len() {
            assert_eq!(parse_layout(&body[..end]), None, "length {end}");
        }
    }

    #[test]
    fn real_dat_menu_layout_records_include_the_command_frame_cursor_and_title() {
        let Some(root) = crate::archive::open_test_install() else {
            return;
        };
        let Some(bytes) = crate::ui_element::read_icon_sheet_unoverlaid(&root) else {
            return;
        };
        for section in walk(&bytes).map(Result::unwrap) {
            if section.kind == ChunkKind::UiMenu as u8 {
                assert!(
                    parse_layout(section.data).is_some(),
                    "section at {}",
                    section.offset
                );
            }
        }
        let layout = find_menu_layout(&bytes, "menu    menuwind").unwrap();
        let frame = &layout.root;
        assert_eq!(
            (frame.x, frame.y, frame.width, frame.height),
            (384, 48, 112, 202)
        );
        assert!(frame
            .resources
            .iter()
            .any(|r| r.tag == 6 && r.group == "anc     anc_s" && r.index == 0));
        let title = layout
            .children
            .iter()
            .find(|row| {
                row.resources
                    .iter()
                    .any(|r| r.tag == 0 && r.group == "menu    comwin" && r.index == 16)
            })
            .unwrap();
        assert_eq!(
            (title.x, title.y, title.offset_x, title.offset_y),
            (0, 0, 19, -3)
        );
    }
}
