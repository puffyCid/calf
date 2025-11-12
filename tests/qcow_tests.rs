use calf::{
    bootsector::boot::PartitionType,
    calf::{CalfReader, CalfReaderAction, QcowInfo},
    format::header::CalfHeader,
};
use ext4_fs::{
    extfs::{Ext4Reader, Ext4ReaderAction},
    structs::{Ext4Hash, FileInfo, FileType},
};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

#[test]
fn test_debian() {
    let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_location.push("tests/test_data/qcow/debian13.qcow");

    let reader = File::open(test_location.to_str().unwrap()).unwrap();
    let buf = BufReader::new(reader);

    let mut calf = CalfReader { fs: buf };

    let info = QcowInfo {
        header: calf.header().unwrap(),
        level1_table: calf.level1_entries().unwrap(),
    };
    let mut os_reader = calf.os_reader(&info).unwrap();
    let boot_info = os_reader.get_boot_info().unwrap();

    os_reader.seek(SeekFrom::Start(1048576 + 1024)).unwrap();
    let mut bytes = vec![0; 1024];

    os_reader.read(&mut bytes).unwrap();
    assert_eq!(
        bytes,
        [
            16, 7, 7, 0, 0, 16, 28, 0, 51, 103, 1, 0, 211, 210, 23, 0, 162, 116, 6, 0, 0, 0, 0, 0,
            2, 0, 0, 0, 2, 0, 0, 0, 0, 128, 0, 0, 0, 128, 0, 0, 144, 31, 0, 0, 7, 32, 17, 105, 7,
            32, 17, 105, 2, 0, 255, 255, 83, 239, 1, 0, 1, 0, 0, 0, 101, 99, 17, 105, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 1, 0, 0, 60, 16, 0, 0, 198, 34, 0,
            0, 107, 4, 1, 0, 190, 150, 98, 8, 161, 251, 65, 43, 182, 134, 229, 195, 189, 110, 224,
            197, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 129, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 251, 180, 59, 108, 210, 207, 69, 50, 163, 254, 14, 214, 136, 156, 145, 149, 1,
            1, 64, 0, 12, 0, 0, 0, 0, 0, 0, 0, 101, 99, 17, 105, 10, 243, 1, 0, 4, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 128, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 32, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 177, 160, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 126, 208, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 77, 116, 88, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 73, 45, 241,
            146
        ]
    );

    assert_eq!(boot_info.partitions.len(), 12);
    for entry in boot_info.partitions {
        if entry.partition_type != PartitionType::Linux {
            continue;
        }
        let os_reader = calf.os_reader(&info).unwrap();

        let test = BufReader::new(os_reader);
        let mut ext4_reader = Ext4Reader::new(test, 4096, entry.offset_start).unwrap();

        let block = ext4_reader.superblock().unwrap();
        if block.filesystem_id == "0b43d8e6-e877-460f-a713-ce9d80ec6904" {
            assert_eq!(block.last_mount_path, "/home");
            let root = ext4_reader.root().unwrap();
            assert_eq!(root.inode, 2);
            assert_eq!(root.children.len(), 3);
            assert_eq!(root.children[1].file_type, FileType::Directory);

            let child = ext4_reader.read_dir(root.children[1].inode).unwrap();
            assert!(!child.name.is_empty());
            assert_ne!(child.created, 0);

            let grand_child = ext4_reader.read_dir(129793).unwrap();
            assert_eq!(grand_child.children.len(), 4);

            let bytes = ext4_reader.read(129794).unwrap();
            assert_eq!(bytes.len(), 3526);
            let hashes = ext4_reader
                .hash(
                    129794,
                    &Ext4Hash {
                        md5: true,
                        sha1: false,
                        sha256: false,
                    },
                )
                .unwrap();
            assert_eq!(hashes.md5, "ee35a240758f374832e809ae0ea4883a");
        } else if block.filesystem_id == "2aae0ee4-0746-41fe-8e16-0552b9aff3ab" {
            assert_eq!(block.last_mount_path, "/tmp");
        } else if block.filesystem_id == "b57a2bc2-596d-4968-9699-53cdafb47b73" {
            assert_eq!(block.last_mount_path, "/var");
            let root = ext4_reader.root().unwrap();
            let mut cache = Vec::new();
            cache.push(root.name.trim_end_matches('/').to_string());

            walk_dir(&mut ext4_reader, &mut cache, &root);
        } else if block.filesystem_id == "be966208-a1fb-412b-b686-e5c3bd6ee0c5" {
            assert_eq!(block.last_mount_path, "/");
        } else {
            panic!("Unknown block info: {block:?} for entry: {entry:?}");
        }
    }
}

fn walk_dir<T: std::io::Seek + std::io::Read>(
    reader: &mut Ext4Reader<T>,
    cache: &mut Vec<String>,
    info: &FileInfo,
) {
    for entry in &info.children {
        if entry.name == "." || entry.name == ".." {
            continue;
        }

        if entry.file_type == FileType::Directory
            && entry.name != "."
            && entry.name != ".."
            && entry.inode != 2
        {
            let child_info = reader.read_dir(entry.inode).unwrap();
            cache.push(child_info.name.trim_matches('/').to_string());

            walk_dir(reader, cache, &child_info);
            cache.pop();
            continue;
        }
        println!(
            "Current file path: {}/{}",
            cache.join("/").replace("//", "/"),
            entry.name
        );
        let test_path = format!("{}/{}", cache.join("/"), entry.name);
        if test_path.contains("wtmp.db") {
            assert_eq!(test_path, "/var/lib/wtmpdb/wtmp.db");
        } else if test_path.contains("emacsen-ispell-default.el") {
            assert_eq!(test_path, "/var/cache/dictionaries-common/emacsen-ispell-default.el")
        }

        let stat = reader.stat(entry.inode).unwrap();
        if entry.file_type == FileType::File && stat.size > 15 {
            // Read 15 bytes of every file
            let mut byte_reader = reader.reader(entry.inode).unwrap();
            let mut buf = [0; 15];
            byte_reader.read(&mut buf).unwrap();
            assert_ne!(buf, [0; 15]);
        }
    }
}
