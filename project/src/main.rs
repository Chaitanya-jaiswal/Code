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
use wg_2024::packet::{Ack, Fragment, FragmentData, Message, MessageContent, Nack, Packet, PacketType};
use game_of_drones::GameOfDrones;
fn main() {}