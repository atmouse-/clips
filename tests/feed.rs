#[cfg(test)]
mod test{
    extern crate clips;

    use byteorder::{BigEndian, WriteBytesExt};

    use clips::runtime::*;
    use clips::storage::Shared;

    use protobuf::Message;

    #[test]
    fn test_push_sel() {
        use tokio::io;
        use tokio::net::TcpStream;
        use tokio::prelude::*;
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open("/tmp/tmp1.png").expect("/tmp/tmp1.png open error");
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        use protobuf::Message;

        let mut magic_size = Vec::new();
        let mut magic = format!("{}\r\n", "").into_bytes();

        let mut sel = ClipMessage::default();
        sel.set_st_name(1);
        sel.set_st_size(data.len() as u32);
        sel.set_st_type(ClipMessage_msgtype::MSG_PUSH);
        sel.set_st_padding(data);

        // let mut magic: Vec<u8> = vec![b'\xfe', b'\xff'];
        
        let mut img = vec!();
        sel.write_to_vec(&mut img).unwrap();
        magic_size.write_u32::<BigEndian>(2 + img.len() as u32).unwrap();

        println!("magic_size: {:?}", &magic_size);
        println!("magic: {:?}", &magic);

        let mut post_data: Vec<u8> = Vec::new();
        post_data.append(&mut magic_size);
        post_data.append(&mut magic);
        post_data.append(&mut img);

        println!("post_data: {}, {:?}", post_data.len(), &post_data);

        let target = "127.0.0.1:9092".parse().unwrap();
        // let target = "127.0.0.1:9092".parse().unwrap();
        let client = TcpStream::connect(&target).and_then(|stream| {
            println!("second stream");

            io::write_all(stream, post_data).then(|result| {
                println!("write to stream; success={:?}", result.is_ok());
                Ok(())
            })
        })
        .map_err(|err| {
            println!("connection error = {:?}", err);
        });

        println!("About to create the stream and write to it...");
        tokio::run(client);
        println!("Stream has been created and writen to.");
    }

    #[test]
    fn test_get_sel() {
        use tokio::io;
        use tokio::net::TcpStream;
        use tokio::prelude::*;
        use tokio::timer::Interval;

        use protobuf::Message;

        let mut sel = ClipMessage::default();
        sel.set_st_name(3);
        sel.set_st_size(0);
        sel.set_st_type(ClipMessage_msgtype::MSG_GET);
        sel.set_st_padding(String::from("").into_bytes());

        // let mut magic: Vec<u8> = vec![b'\xfe', b'\xff'];
        // let mut magic = String::from("\r\n").into_bytes();
        let mut img = Vec::new();
        sel.write_to_vec(&mut img).unwrap();
        // magic.append(&mut img);

        let target = "127.0.0.1:9091".parse().unwrap();
        // let target = "127.0.0.1:9091".parse().unwrap();
        let client = TcpStream::connect(&target)
        .and_then(|stream| {
            println!("second stream");

            io::write_all(stream, img)
        })
        .and_then(|(stream, buf)| {
            // writed buffer
            println!("send buf: {:?}", buf);

            let buf: Vec<u8> = Vec::new();
            // let mut buf = vec![0; 8];
            io::read_to_end(stream, buf)
            // io::read_to_end(reader, buf).then(|result| {
            //     let result = result.unwrap();
            //     println!("read to stream; success={:?}", result.1);
            //     Ok(())
            // })
            // io::read_to_end(reader, buf).and_then(|(_, result)| {
            //     println!("read to stream; success={:?}", result);
            //     Ok(())
            // })

            // tokio::spawn(task);
            // println!("read: {:?}", &buf);
            // Ok(())
            // io::read_to_end(reader, buf)
            // .then(|result| {
            //     println!("read to stream; success={:?}", result.is_ok());
            //     Ok(())
            // })
            // Ok(())
        })
        .and_then(|(_stream, buf)| {
            println!("read to stream; success={:?}", &buf);
            Ok(())
        })
        .map_err(|err| {
            println!("connection error = {:?}", err);
        });

        println!("About to create the stream and write to it...");
        tokio::run(client);
        println!("Stream has been created and writen to.");
    }
}
