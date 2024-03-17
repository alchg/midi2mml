use std::env;
use std::fs::File;
use std::io::Read;

mod converter;
use converter::Converter;

fn read_file(buffer: &mut Vec<u8>) -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        let error_message: String = format!("Usage: {} <midi_filepath>", args[0]);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            error_message,
        ));
    }

    let file_path: &String = &args[1];

    let mut file: File = File::open(file_path)?;
    file.read_to_end(buffer)?;

    Ok(())
}

fn main() {
    let mut data: Vec<u8> = Vec::new();
    let mut converter: Converter;

    if let Err(err) = read_file(&mut data) {
        eprintln!("{}", err);
        std::process::exit(termination::EXIT_FAILURE);
    }

    converter = Converter::new(data);
    if let Err(msg) = converter.convert() {
        eprintln!("{}", msg);
        std::process::exit(termination::EXIT_FAILURE);
    }
}
