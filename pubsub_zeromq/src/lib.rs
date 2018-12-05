// CITA
// Copyright 2016-2017 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate logger;
extern crate zmq;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

pub fn start_zeromq(
    name: &str,
    keys: Vec<String>,
    tx: Sender<(String, Vec<u8>)>,
    rx: Receiver<(String, Vec<u8>)>,
) {
    let context = zmq::Context::new();
    //pub
    let publisher = context.socket(zmq::PUB).unwrap();
    match name {
        "net" => assert!(publisher.bind("tcp://*:6000").is_ok()),
        "chain" => assert!(publisher.bind("tcp://*:6001").is_ok()),
        "jsonrpc" => assert!(publisher.bind("tcp://*:6002").is_ok()),
        "consensus" => assert!(publisher.bind("tcp://*:6003").is_ok()),
        "executor" => assert!(publisher.bind("tcp://*:6004").is_ok()),
        "auth" => assert!(publisher.bind("tcp://*:6005").is_ok()),
        "snapshot" => assert!(publisher.bind("tcp://*:6006").is_ok()),
        "synchronizer" => {},
        _ => error!("not hava {} module !", name),
    }

    let _ = thread::Builder::new()
        .name("publisher".to_string())
        .spawn(move || loop {
            let ret = rx.recv();

            if ret.is_err() {
                break;
            }
            let (topic, msg) = ret.unwrap();
            warn!("start_zeromq send topic {}",topic);
            publisher
                .send_multipart(&[&(topic.into_bytes())], zmq::SNDMORE)
                .unwrap();
            publisher.send(&msg, 0).unwrap();
        });

    //sub

    let network_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(network_subscriber.connect("tcp://localhost:6000").is_ok());

    let chain_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(chain_subscriber.connect("tcp://localhost:6001").is_ok());

    let jsonrpc_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(jsonrpc_subscriber.connect("tcp://localhost:6002").is_ok());

    let consensus_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(consensus_subscriber.connect("tcp://localhost:6003").is_ok());

    let executor_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(executor_subscriber.connect("tcp://localhost:6004").is_ok());

    let auth_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(auth_subscriber.connect("tcp://localhost:6005").is_ok());

    let snapshot_subscriber = context.socket(zmq::SUB).unwrap();
    assert!(snapshot_subscriber.connect("tcp://localhost:6006").is_ok());

    let mut flag = 0;
    for topic in keys {
        let tmp = topic.clone();
        let v: Vec<&str> = tmp.split('.').collect();

        warn!("pubsub  topic {:?} vector {:?} ",tmp,v);
        flag = match v[0] {
            "net" => {
                network_subscriber
                    .set_subscribe(&topic.into_bytes())
                    .unwrap();
                flag | 0x01
            }
            "chain" => {
                chain_subscriber
                    .set_subscribe(&topic.to_string().into_bytes())
                    .unwrap();
                flag | 0x02
            }
            "jsonrpc" => {
                jsonrpc_subscriber
                    .set_subscribe(&topic.to_string().into_bytes())
                    .unwrap();
                flag | 0x04
            }
            "consensus" => {
                consensus_subscriber
                    .set_subscribe(&topic.to_string().into_bytes())
                    .unwrap();
                flag | 0x08
            }
            "executor" => {
                executor_subscriber
                    .set_subscribe(&topic.to_string().into_bytes())
                    .unwrap();
                flag | 0x10
            }
            "auth" => {
                auth_subscriber
                    .set_subscribe(&topic.to_string().into_bytes())
                    .unwrap();
                flag | 0x20
            }
            "snapshot" => {
                snapshot_subscriber
                    .set_subscribe(&topic.to_string().into_bytes())
                    .unwrap();
                flag | 0x40
            }
            "synchronizer" => {
                flag
            }
            _ => {
                error!("invalid  flag! topic {}",tmp);
                flag
            }
        }
    }


    if flag & 0x01 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("network_subscriber".to_string())
            .spawn(move || loop {
                let topic = network_subscriber.recv_string(0).unwrap().unwrap();
                let msg = network_subscriber.recv_bytes(0).unwrap();
                warn!("network_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }

    if flag & 0x02 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("chain_subscriber".to_string())
            .spawn(move || loop {
                let topic = chain_subscriber.recv_string(0).unwrap().unwrap();
                let msg = chain_subscriber.recv_bytes(0).unwrap();
                warn!("chain_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }

    if flag & 0x04 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("jsonrpc_subscriber".to_string())
            .spawn(move || loop {
                let topic = jsonrpc_subscriber.recv_string(0).unwrap().unwrap();
                let msg = jsonrpc_subscriber.recv_bytes(0).unwrap();
                warn!("jsonrpc_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }

    if flag & 0x08 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("consensus_subscriber".to_string())
            .spawn(move || loop {
                let topic = consensus_subscriber.recv_string(0).unwrap().unwrap();
                let msg = consensus_subscriber.recv_bytes(0).unwrap();
                warn!("consensus_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }

    if flag & 0x10 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("executor_subscriber".to_string())
            .spawn(move || loop {
                let topic = executor_subscriber.recv_string(0).unwrap().unwrap();
                let msg = executor_subscriber.recv_bytes(0).unwrap();
                warn!("executor_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }

    if flag & 0x20 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("auth_subscriber".to_string())
            .spawn(move || loop {
                let topic = auth_subscriber.recv_string(0).unwrap().unwrap();
                let msg = auth_subscriber.recv_bytes(0).unwrap();
                warn!("auth_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }

    if flag & 0x40 != 0 {
        let other_tx = tx.clone();
        let _ = thread::Builder::new()
            .name("snapshot_subscriber".to_string())
            .spawn(move || loop {
                let topic = snapshot_subscriber.recv_string(0).unwrap().unwrap();
                let msg = snapshot_subscriber.recv_bytes(0).unwrap();
                warn!("snapshot_subscriber  recived {}  ",topic);
                let _ = other_tx.send((topic, msg));
            });
    }
}
