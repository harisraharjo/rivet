// use std::io::{BufRead, Cursor, Read, Seek, SeekFrom, Write};

// pub struct Loader;
// impl Loader {
//     // fn new() -> Loader {
//     //     Loader{  }
//     // }

//     fn write_ten_bytes_at_end<W: Write + Seek>(mut writer: W) -> std::io::Result<()> {
//         writer.seek(SeekFrom::End(-10))?;

//         for i in 0..10 {
//             writer.write(&[i])?;
//         }

//         // all went well
//         Ok(())
//     }
// }
