use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, Serializer},
};
use std::fmt;
use std::io::Read;
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender, TryRecvError};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct DeNBTBlob(pub nbt::Blob);

impl Serialize for DeNBTBlob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DeNBTBlob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // this only works because my own deserializer doesnt look at the len param
        deserializer.deserialize_tuple(0, VisitorImpl)
    }
}

struct VisitorImpl;

impl<'de> Visitor<'de> for VisitorImpl {
    type Value = DeNBTBlob;

    fn expecting(&self, _: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let (tx_byte, rx_byte) = sync_channel::<u8>(0);
        let (tx_ask, rx_ask) = sync_channel::<()>(0);
        let (tx_result, rx_result) = channel::<nbt::Result<nbt::Blob>>();
        let reader = ReadImpl(rx_byte, tx_ask);

        let child = thread::spawn(move || {
            let mut reader = reader;
            let result = nbt::Blob::from_reader(&mut reader);
            tx_result.send(result).unwrap();
        });

        let result = loop {
            if let Ok(res) = rx_result.try_recv() {
                break Some(res);
            }

            match rx_ask.try_recv() {
                Ok(_) => {
                    if let Some(byte) = seq.next_element().unwrap() {
                        tx_byte.send(byte).unwrap();
                    } else {
                        // we ran out of data
                        // need to kill child process idk
                        break None;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    eprintln!("disconnected");
                    break None;
                }
                Err(TryRecvError::Empty) => {}
            }

            thread::yield_now();
        };

        child.join().unwrap();

        let result = result
            .expect("eof while reading nbt")
            .expect("error from nbt::Blob");

        Ok(DeNBTBlob(result))
    }
}

struct ReadImpl(Receiver<u8>, SyncSender<()>);

impl Read for ReadImpl {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        self.1.send(()).unwrap();
        buf[0] = self.0.recv_timeout(Duration::from_secs(1)).unwrap();

        Ok(1)
    }
}
