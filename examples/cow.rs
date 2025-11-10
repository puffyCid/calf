use calf::{
    bootsector::boot::PartitionType,
    calf::{CalfReader, CalfReaderAction, QcowInfo},
    format::{header::CalfHeader, level::CalfLevel},
};
use ext4_fs::{
    extfs::{Ext4Reader, Ext4ReaderAction},
    structs::Ext4Hash,
};
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
        // println!("Require QCOW input file!!")
        qcow_info("/home/ubunty/Downloads/debian12-uni");
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
    let size = header.level_one_table_ref * 8;
    let levels = calf.levels(&header.level_one_table_offset, &size).unwrap();
    println!("{}", header.level_one_table_offset);
    println!("size: {size}, levels count: {}", levels.len());
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
    assert_eq!(
        bytes,
        [
            160, 72, 23, 0, 0, 33, 93, 0, 12, 168, 4, 0, 223, 26, 72, 0, 145, 231, 20, 0, 0, 0, 0,
            0, 2, 0, 0, 0, 2, 0, 0, 0, 0, 128, 0, 0, 0, 128, 0, 0, 224, 31, 0, 0, 30, 15, 245, 104,
            28, 15, 245, 104, 4, 0, 255, 255, 83, 239, 1, 0, 1, 0, 0, 0, 234, 136, 129, 104, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 1, 0, 0, 60, 0, 0, 0, 198, 2,
            0, 0, 107, 4, 0, 0, 231, 81, 59, 80, 214, 173, 75, 41, 157, 37, 178, 180, 91, 99, 70,
            220, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 156,
            248, 13, 0, 25, 111, 113, 171, 110, 15, 78, 193, 135, 101, 28, 109, 104, 200, 93, 235,
            1, 1, 64, 0, 12, 0, 0, 0, 0, 0, 0, 0, 234, 136, 129, 104, 10, 243, 1, 0, 4, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 4, 132, 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 32, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 241, 211, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 44, 34, 2, 0, 0, 0,
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
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 166, 216, 123,
            157
        ]
    );

    println!("All root directory info for each partition. Total: {}", boot_info.partitions.len());
    for entry in boot_info.partitions {
        if entry.partition_type != PartitionType::Linux {
            continue;
        }
        let mut os_reader = calf.os_reader(&info).unwrap();

        let test = BufReader::new(os_reader);

        println!("entry: {entry:?}");

        //let mut ext4_reader = Ext4Reader::new(test, 4096, entry.offset_start).unwrap();
        // println!("Superblock: {:?}", ext4_reader.superblock().unwrap());
    }
}
