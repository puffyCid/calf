use calf::{
    calf::{CalfReader, CalfReaderAction, QcowInfo},
    format::{header::CalfHeader, level::CalfLevel},
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
    let info = os_reader.get_boot_info().unwrap();
    println!("Boot info: {info:?}");

    os_reader.seek(SeekFrom::Start(1048576)).unwrap();
    let mut bytes = vec![0; 1024];

    os_reader.read_exact(&mut bytes).unwrap();
    println!("First 1024 bytes: {bytes:?}");
}
