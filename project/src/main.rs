use crossbeam_channel::*;
use wg_2024::config::{Config,Server,Client,Drone};
use wg_2024::controller::Command;
use wg_2024::drone::{ DroneOptions};
use wg_2024::network::topology::{self, Node, NodeRef, NodeType, ServerType, Topology};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::time::Instant;
use wg_2024::packet::{Ack, Fragment, Message, MessageContent, Nack, Packet, PacketType};
use game_of_drones::GameOfDrones;
fn main() {
    let config_data = fs::read_to_string("config.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    println!("{:#?}", config);
    let s = config.server;
    let c = config.client;
    let d = config.drone;
    
    let mut top = Topology { nodes: construct_network(&d, c, s)};

    let vv = build_drone_options(&top.nodes,&d); 

    let drones_top = MyTop::new(vv);

    for dd in &d {
        for ddd in  &drones_top.drones{
            if dd.id as u8 == ddd.borrow_mut().id {
                for k in &dd.connected_node_ids {
                    ddd.borrow_mut().packet_send.insert(*k as u8, unbounded::<Packet>().0);
                }
            }
        }
    }

    let res = send_n_rec(1, 2, &drones_top);
    if res.is_ok() {
        println!("OK");
    } else {
        match res.unwrap_err().0.pack_type {
            PacketType::Ack(s)=>{
                println!("ACK {:?}",s.time_received);
            },
            PacketType::Nack(_s)=>{
                println!("NACK");
            },
            PacketType::MsgFragment(_s)=>{
                println!("MACK");
            },
            _=>{}
        }
    }   
}


fn construct_network(
    drones: &Vec<Drone>,
    clients: Vec<Client>,
    servers: Vec<Server>,
) -> Vec<NodeRef> {
    // Create a HashMap for quick lookup of nodes by their ID
    let mut node_map: HashMap<NodeId, NodeRef> = HashMap::new();

    // Helper closure to add a node to the map
    let mut add_node = |id: NodeId, node_type: NodeType| -> NodeRef {
        let node = Rc::new(RefCell::new(Node {
            name: id,
            node_type,
            neighbors: HashMap::new(),
        }));
        node_map.insert(id, Rc::clone(&node));
        node
    };

    // Add drones
    for drone in drones {
        add_node(drone.id as u8, NodeType::Drone(drone.id as u8));
    }

    // Add clients
    for client in &clients {
        add_node(client.id as u8, NodeType::Client(client.id as u8));
    }

    // Add servers
    for server in &servers {
        add_node(server.id as u8, NodeType::Server(ServerType::Chat, server.id as u8)); // Assuming Chat for example
    }

    // Populate neighbors
    for drone in drones {
        if let Some(node) = node_map.get(&(drone.id as u8)) {
            for &neighbor_id in &drone.connected_node_ids {
                if let Some(neighbor) = node_map.get(&(neighbor_id as u8)) {
                    node.borrow_mut().neighbors.insert(neighbor_id as u8, Rc::clone(neighbor));
                }
            }
        }
    }

    for client in clients {
        if let Some(node) = node_map.get(&(client.id as u8)) {
            for &neighbor_id in &client.connected_drone_ids {
                if let Some(neighbor) = node_map.get(&(neighbor_id as u8)) {
                    node.borrow_mut().neighbors.insert(neighbor_id as u8, Rc::clone(neighbor));
                }
            }
        }
    }

    for server in servers {
        if let Some(node) = node_map.get(&(server.id as u8)) {
            for &neighbor_id in &server.connected_drone_ids {
                if let Some(neighbor) = node_map.get(&(neighbor_id as u8)) {
                    node.borrow_mut().neighbors.insert(neighbor_id as u8, Rc::clone(neighbor));
                }
            }
        }
    }

    // Return all nodes as a Vec<NodeRef>
    node_map.into_values().collect()
}


pub fn build_drone_options(nodes: &Vec<Rc<RefCell<Node>>>,drones: &Vec<Drone>) -> Vec<DroneOptions> {
    let mut drone_options = Vec::new();

    for node in nodes {
        let node_borrow = node.borrow();

        // Only process drones
        if let NodeType::Drone(drone_id) = node_borrow.node_type {
            // Create crossbeam channels
            let (sim_contr_send, sim_contr_recv) = unbounded::<Command>();
            let (packet_send, packet_recv) = unbounded::<Packet>();

            let mut pdr: f32 = 0.05;
            for ds in drones {
                if ds.id as u8 == drone_id {
                    pdr = ds.pdr as f32;
                }
            }
            // Create DroneOptions and push to the result vector
            drone_options.push(DroneOptions {
                id: drone_id,
                sim_contr_send,
                sim_contr_recv,
                packet_recv,
                pdr
            });
        }
    }

    drone_options
}

pub struct MyTop {
    drones: Vec<Rc<RefCell<GameOfDrones>>>
}

impl MyTop {
    pub fn new (options: Vec<DroneOptions>)->Self{
        let mut v = Vec::new();
        for op in options {
            v.push(Rc::new(RefCell::new(GameOfDrones::new(op))));
        }
        MyTop { drones: v }
    }
}

pub fn send_n_rec (start: NodeId, end: NodeId, top: &MyTop )->Result<(),SendError<Packet>>{
    let mut res = Ok(());
    for ids in &top.drones {
        if  ids.borrow().id == start {
            for s in &ids.borrow().packet_send  {
                if *s.0 == end {
                    res = s.1.send(Packet { 
                        pack_type: PacketType::Ack(Ack { fragment_index: 1, time_received: Instant::now() }), 
                        routing_header: SourceRoutingHeader { hop_index: 1, hops: Vec::new() }, 
                        session_id: 1 
                    });
                }
            }
        }
    };
    if res.is_err() {
        return Err(SendError(Packet{pack_type:PacketType::Ack(Ack{fragment_index:10,time_received:Instant::now()}),
                                            routing_header:SourceRoutingHeader { hop_index: 1, hops: Vec::new() },session_id:111}));
    }

    for ids in &top.drones {
        if  ids.borrow().id == end {
            let rec = ids.borrow().packet_recv.recv();
            if rec.is_ok(){
                return Ok(());
            } else {
                return Err(SendError(Packet{pack_type:PacketType::Nack(Nack{fragment_index:10,time_of_fail:Instant::now(),nack_type: wg_2024::packet::NackType::Dropped}),
                                            routing_header:SourceRoutingHeader { hop_index: 1, hops: Vec::new() },session_id:1}));
            }
        }
    }
    return Ok(());
}