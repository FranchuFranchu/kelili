/// Lazy implementation of the Kademlia protocol.
use rand::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    time::Duration,
};
use tokio::sync::mpsc::{self, channel};

use blake2::Digest;

pub type Hashed = [u8; 64];

pub use super::types::Id;

#[derive(Clone, Debug)]
pub enum MessageData {
    Ping {
        id: u64,
        time: u64,
    },
    Pong {
        id: u64,
        time: u64,
    },
    Find {
        id: u64,
        hash: Id,
    },
    FoundPeers {
        id: u64,
        peers: Vec<PeerInfo>,
    },
    FoundData {
        id: u64,
        data: Box<[u8]>,
        propagate: bool,
    },
    Stop,
}

#[derive(Clone, Debug)]
pub struct Message {
    from: PeerInfo,
    contents: MessageData,
}

#[derive(Clone)]
pub struct PeerInfo {
    tx: mpsc::Sender<Message>,
    id: Id,
}
use core::fmt::Debug;
impl Debug for PeerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", encode_id(&self.id)))
    }
}

impl PeerInfo {
    pub async fn send_peer_info(
        &self,
        from: &PeerInfo,
        other: &PeerInfo,
    ) -> Result<(), Box<dyn Error>> {
        Ok(self
            .tx
            .send(Message {
                from: from.clone(),
                contents: MessageData::FoundPeers {
                    id: 0,
                    peers: vec![other.clone()],
                },
            })
            .await?)
    }
    pub async fn send_find(&self, from: &PeerInfo, hash: &Id) -> Result<(), Box<dyn Error>> {
        Ok(self
            .tx
            .send(Message {
                from: from.clone(),
                contents: MessageData::Find {
                    id: 0,
                    hash: hash.clone(),
                },
            })
            .await?)
    }
}

#[derive(Debug)]
pub struct Peer {
    store: HashMap<Id, Box<[u8]>>,
    k: u32,
    buckets: [Vec<PeerInfo>; 256],
    peer_distance: HashMap<Id, i64>,
    msg_sent_at: HashMap<u64, u64>,
    waiting_finds: HashMap<u64, (Id, mpsc::Sender<Option<(Id, Box<[u8]>)>>, u8)>,
    m_id_to_find_id: HashMap<u64, u64>,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
    id: Id,
    rng: Box<dyn N>,
}
trait N: rand::RngCore + rand::CryptoRng + core::fmt::Debug + Send {}
impl<T> N for T where T: rand::RngCore + rand::CryptoRng + core::fmt::Debug + Send {}

pub fn encode_id(id: &Id) -> String {
    format!("0x{}", &hex::encode(id)[..4])
}

pub fn xor_distance(from: &Id, other: &Id) -> Id {
    std::array::from_fn(|i| from[i] ^ other[i])
}

impl Peer {
    pub fn hash(&self, data: &[u8]) -> Id {
        let mut hasher = blake2::Blake2s256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
    pub fn distance_to(&self, other: &Id) -> Id {
        xor_distance(&self.id, other)
    }
    pub fn bucket_num(&mut self, id: &Id) -> Option<usize> {
        for (idx, i) in id.iter().map(|x| 8 - x.leading_zeros()).enumerate() {
            if i > 0 {
                return Some(idx * 8 + i as usize);
            }
        }
        // This is ourselves.
        return None;
    }
    pub fn find_closest_peers(&mut self, hash: &Id, amount: &u32) -> Vec<PeerInfo> {
        self.buckets
            .iter()
            .map(|x| x.iter())
            .flatten()
            .fold(
                BTreeMap::from([(xor_distance(&self.id, hash), self.info())]),
                |mut acc, i| {
                    acc.insert(xor_distance(&i.id, hash), i.clone());
                    if acc.len() > *amount as usize {
                        acc.pop_last();
                    }
                    acc
                },
            )
            .values()
            .cloned()
            .collect()
    }
    pub async fn retry_find(&mut self, find_id: u64) -> Result<(), Box<dyn Error>> {
        let (hash, send_to, _ttl) = self.waiting_finds.get(&find_id).unwrap().clone();
        let peers = self.find_closest_peers(&hash, &3);
        if peers.len() == 1 {
            send_to.send(None).await?;
            return Ok(());
        }
        for peer in peers {
            if peer.id != self.id {
                let m_id = self.rng.next_u64();
                self.m_id_to_find_id.insert(m_id, find_id);
                peer.tx
                    .send(Message {
                        from: self.info(),
                        contents: MessageData::Find {
                            id: m_id,
                            hash: hash.clone(),
                        },
                    })
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn find_with_ttl(
        &mut self,
        hash: &Id,
        send_to: &mpsc::Sender<Option<(Id, Box<[u8]>)>>,
        ttl: &u8,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(data) = self.store.get(hash) {
            send_to.send(Some((hash.clone(), data.clone()))).await?;
            return Ok(());
        }
        let f_id = self.rng.next_u64();
        self.waiting_finds
            .insert(f_id, (hash.clone(), send_to.clone(), *ttl));
        self.retry_find(f_id).await
    }
    pub async fn find(&mut self, hash: &Id) -> Result<Option<Box<[u8]>>, Box<dyn Error>> {
        let (tx, mut rx) = channel(100);
        self.find_with_ttl(hash, &tx, &5).await?;
        let self_tx = self.tx.clone();
        let stop_msg = self.make_msg(MessageData::Stop);
        // TODO: Blocks. Better way?
        let (tx2, mut rx2) = channel(100);
        tokio::spawn(async move {
            let m = rx.recv().await;
            let _ = self_tx.send(stop_msg).await;
            tx2.send(m).await.unwrap();
        });
        self.run().await.unwrap();
        let v = rx2.recv().await.unwrap().unwrap();
        Ok(v.map(|x| x.1))
    }
    pub fn add_peer(&mut self, info: &PeerInfo) {
        let dist = self.distance_to(&info.id);
        if let Some(idx) = self.bucket_num(&dist) {
            let bucket = &mut self.buckets[idx];
            if !bucket.iter().any(|x| x.id == info.id) {
                bucket.push(info.clone());
                if bucket.len() > self.k as usize {
                    let id = self.id;
                    let max_item = bucket
                        .iter()
                        .enumerate()
                        .max_by_key(|k| xor_distance(&id, &k.1.id))
                        .unwrap();
                    bucket.remove(max_item.0);
                }
            }
        }
    }
    pub fn make_msg(&mut self, msg: MessageData) -> Message {
        Message {
            from: self.info(),
            contents: msg,
        }
    }
    pub async fn handle_msg(&mut self, msg: Message) -> Result<(), Box<dyn Error>> {
        // println!("{} <- {} {:?}", encode_id(&self.id), encode_id(&msg.from.id), msg.contents);
        self.add_peer(&msg.from);
        match msg.contents {
            MessageData::Ping { id, time: _time } => {
                msg.from
                    .tx
                    .send(Message {
                        from: self.info(),
                        contents: MessageData::Pong { id, time: 0 },
                    })
                    .await?;
            }
            MessageData::Pong { id, time } => {
                if let Some(send_time) = self.msg_sent_at.remove(&id) {
                    self.peer_distance
                        .insert(msg.from.id, (time as i64) - (send_time as i64));
                }
            }
            MessageData::Find { id, hash } => {
                if let Some(data) = self.store.get(&hash) {
                    msg.from
                        .tx
                        .send(Message {
                            from: self.info(),
                            contents: MessageData::FoundData {
                                id,
                                data: data.clone(),
                                propagate: false,
                            },
                        })
                        .await?;
                } else {
                    // Return closest peers
                    msg.from
                        .tx
                        .send(Message {
                            from: self.info(),
                            contents: MessageData::FoundPeers {
                                id,
                                peers: self.find_closest_peers(&hash, &self.k.clone()),
                            },
                        })
                        .await?;
                }
            }
            MessageData::FoundPeers { id, peers } => {
                let mut found_self = 0;
                for peer in peers {
                    if peer.id == self.id {
                        found_self += 1;
                    }
                    self.add_peer(&peer);
                }
                if let Some((expect_hash, send_to, ttl)) = self
                    .m_id_to_find_id
                    .remove(&id)
                    .map(|x| self.waiting_finds.remove(&x))
                    .flatten()
                {
                    if ttl > 0 {
                        self.find_with_ttl(&expect_hash, &send_to, &(ttl - found_self))
                            .await?;
                    } else {
                        // Not found; exceeded TTL.
                        send_to.send(None).await?;
                    }
                } else {
                    // Not called for
                };
            }
            MessageData::FoundData {
                id,
                data,
                propagate,
            } => {
                if let Some((expect_hash, send_to, _)) = self
                    .m_id_to_find_id
                    .remove(&id)
                    .map(|x| self.waiting_finds.remove(&x))
                    .flatten()
                {
                    assert!(expect_hash == self.hash(&data));
                    send_to.send(Some((expect_hash, data.clone()))).await?;
                }
                if propagate {
                    self.store(data).await?;
                }
            }
            MessageData::Stop => self.rx.close(),
        };
        Ok(())
    }
    pub async fn store(&mut self, data: Box<[u8]>) -> Result<(), Box<dyn Error>> {
        let h = self.hash(&data);
        // println!("{} Propg {} | {} = {}", encode_id(&self.id), encode_id(&self.distance_to(&h)), encode_id(&h), hex::encode(&data));
        self.store.insert(h, data.clone());
        let peer = self
            .find_closest_peers(&h, &1)
            .first()
            .cloned()
            .expect("No peers!");
        if peer.id != self.id {
            let id = self.rng.next_u64();
            peer.tx
                .send(self.make_msg(MessageData::FoundData {
                    id,
                    data,
                    propagate: true,
                }))
                .await?;
        } else {
            // println!("{} Store {} | {} = {}", encode_id(&self.id), encode_id(&self.distance_to(&h)), encode_id(&h), hex::encode(data));
        }
        Ok(())
    }
    pub async fn run_timeout(&mut self, duration: Duration) -> Result<(), Box<dyn Error>> {
        while let Ok(Some(msg)) = tokio::time::timeout(duration, self.rx.recv()).await {
            self.handle_msg(msg).await?;
        }
        Ok(())
    }
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(msg) = self.rx.recv().await {
            self.handle_msg(msg).await?;
        }
        Ok(())
    }
    pub fn new(rng: &mut dyn rand::RngCore) -> Peer {
        let (tx, rx) = mpsc::channel(100);

        Peer {
            store: HashMap::new(),
            k: 20,
            buckets: std::array::from_fn(|_x| Vec::new()),
            rx,
            tx,
            id: std::array::from_fn(|_x| (rng.next_u32() & 0xFF) as u8),
            msg_sent_at: HashMap::new(),
            peer_distance: HashMap::new(),
            waiting_finds: HashMap::new(),
            m_id_to_find_id: HashMap::new(),
            rng: Box::new(rand_chacha::ChaCha20Rng::seed_from_u64(rng.next_u64())),
        }
    }
    pub fn info(&self) -> PeerInfo {
        PeerInfo {
            id: self.id.clone(),
            tx: self.tx.clone(),
        }
    }
}
