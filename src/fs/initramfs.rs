use spin::lazylock::LazyLock;

use crate::{
    boot::info::MODULES,
    fs::tar::{BLOCK, Entry, Tar},
};

// Skip the "initramfs" word from path. Example, "initramfs/bin/init" -> "/bin/init"
const PATH_SKIP: usize = 9;

pub static INITRAMFS: LazyLock<Tar> = LazyLock::new(|| {
    let module = MODULES
        .modules()
        .iter()
        .next()
        .expect("Module is empty, can't parse tar archive");

    let data = module.data();
    let mut offset = 0usize;

    let mut tar = Tar::new();

    while (offset + BLOCK) < data.len() {
        let end_header = offset + BLOCK;

        let header_block = &data[offset..end_header];
        if header_block.iter().all(|b| *b == 0) {
            break;
        }

        let name = str::from_utf8(&header_block[PATH_SKIP..100])
            .expect("Failed to parse filename")
            .trim_matches('\0');

        let size = {
            let mut value = 0u64;

            for &byte in &header_block[124..136] {
                if byte == 0 || byte == b' ' {
                    break;
                }

                value = value * 8 + (byte - b'0') as u64;
            }

            value
        };

        let data = &data[end_header..(end_header + size as usize)];

        tar.push(Entry::new(name, data, size));

        offset += (BLOCK + size as usize + 511) & !511;
    }

    tar
});
