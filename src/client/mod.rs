use clap::ValueEnum;
use futures::{AsyncWriteExt, io::BufWriter};
use smol::net::unix::UnixStream;

pub fn activate(item: Item) {
    smol::block_on(async {
        let stream = UnixStream::connect(crate::daemon::SOCKET_PATH.as_path())
            .await
            .unwrap();
        match item {
            Item::PowerProfile => {
                let mut stream = BufWriter::new(stream);
                stream.write_all(b"activate/power_profile").await.unwrap();
                stream.close().await.unwrap();
            }
            _ => todo!(),
        }
    });
}

#[derive(Clone, ValueEnum)]
pub enum Item {
    Volume,
    Media,
    MonitorBrightness,
    PowerProfile,
}
