use calf::{
    bootsector::boot::PartitionType,
    calf::{CalfReader, CalfReaderAction, QcowInfo},
    format::{header::CalfHeader, level::CalfLevel},
};
use ext4_fs::extfs::{Ext4Reader, Ext4ReaderAction};
use std::{
    env,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

fn main() {
    println!("Lets get some basic QCOW info!\n");

    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let path = &args[1];
        if Path::new(path).is_file() {
            qcow_info(path);
        } else {
            println!("This is not a file")
        }
    } else {
        println!("Require QCOW input file!!")
    }
}

fn qcow_info(path: &str) {
    let reader = File::open(path).unwrap();
    let buf = BufReader::new(reader);
    let mut calf = CalfReader { fs: buf };
    let header = calf.header().unwrap();

    println!(
        "Version: {} - OS Size: {} - Compression Support: {:?}",
        header.version, header.size, header.compression_method
    );

    println!("{header:?}");
    let size = header.level_one_table_ref;
    let levels = calf.levels(header.level_one_table_offset, size).unwrap();
    println!("{}", header.level_one_table_offset);
    println!("size: {size}, levels count: {}\n\n", levels.len());

    println!("Extensions: {:?}", calf.extensions().unwrap());

    let info = QcowInfo {
        header: calf.header().unwrap(),
        level1_table: calf.level1_entries().unwrap(),
    };
    let mut os_reader = calf.os_reader(&info).unwrap();
    println!("OS size is {} bytes", os_reader.get_os_size());
    let boot_info = os_reader.get_boot_info().unwrap();
    println!("Boot info: {boot_info:?}");

    os_reader.seek(SeekFrom::Start(1048576 + 1024)).unwrap();
    let mut bytes = vec![0; 1024];

    os_reader.read(&mut bytes).unwrap();

    println!(
        "All root directory info for each partition. Total: {}",
        boot_info.partitions.len()
    );
    for entry in boot_info.partitions {
        if entry.partition_type != PartitionType::Linux {
            continue;
        }
        let os_reader = calf.os_reader(&info).unwrap();

        let test = BufReader::new(os_reader);

        println!("entry: {entry:?}");

        let mut ext4_reader = Ext4Reader::new(test, 4096, entry.offset_start).unwrap();
        println!("Superblock: {:?}", ext4_reader.superblock().unwrap());
        let root = ext4_reader.root().unwrap();

        println!("root info: {root:?}");
        println!("Root children...\n\n");
        for value in root.children {
            let stat_value = ext4_reader.stat(value.inode).unwrap();
            println!("Stat info for '{}': :{stat_value:?}\n\n", value.name);
        }
    }
}
