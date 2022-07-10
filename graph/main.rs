use generational_arena::{Arena, Index};
use std::vec::Vec;

#[derive(Debug)]
pub enum RoadNodeType {
    COMMERCIAL,
    INDUSTRIAL,
    OFFICE,
    RESIDENTIAL
}

#[derive(Debug)]
pub struct RoadNode {
    pub id: String,
    pub node_type: RoadNodeType,
    pub supply: usize,
    neighbors: Vec<Index>
}

impl RoadNode {
    pub fn new(id: String, node_type: RoadNodeType, supply: usize) -> RoadNode {
        RoadNode { id, node_type, supply, neighbors: vec![] }
    }

    pub fn new_with_neighbors(id: String, node_type: RoadNodeType, supply: usize, neighbors: Vec<Index>) -> RoadNode {
        RoadNode { id, node_type, supply, neighbors }
    }

    pub fn add_neighbor(&mut self, neighbor: Index) {
        self.neighbors.push(neighbor);
    }

    pub fn remove_neighbor(&mut self, neighbor_to_delete: &Index) {
        self.neighbors.retain(|neighbor| {
            neighbor_to_delete == neighbor
        })
    }
}

pub struct RoadNetwork {
    road_arena: Arena<RoadNode>
}

impl RoadNetwork {
    pub fn new() -> RoadNetwork {
        RoadNetwork { road_arena: Arena::new() }
    }

    pub fn add_node(&mut self, id: String, node_type: RoadNodeType, supply: usize) -> Index {
        self.road_arena.insert(RoadNode::new(id, node_type, supply))
    }

    pub fn add_node_with_neighbors(&mut self, id: String, node_type: RoadNodeType, supply: usize, neighbors: Vec<Index>) -> Index {
        let copied_neighbors = neighbors.clone();
        let new_index = self.road_arena.insert(RoadNode::new_with_neighbors(id, node_type, supply, neighbors));
        copied_neighbors.iter().for_each(|node| {
            let current_node_option = self.road_arena.get_mut(*node);
            if let Some(current_node) = current_node_option {
                current_node.add_neighbor(new_index.clone())
            }
        });
        new_index
    }

    pub fn remove_node(&mut self, index: Index) -> Option<RoadNode> {
        self.road_arena.remove(index)
    }

    fn get_dunkards_path(&self, start_index: &Index, stop_index: &Index) -> Result<Vec<&Index>,i8> {
        let mut visited = vec![start_index];
        let mut path = Vec::<&Index>::new();
        let mut queue = Vec::<&Index>::new();
        // The first check
        if self.road_arena.get(*start_index).is_none() {
            return Err(4);
        }

        // If start is end return immediately
        if start_index == stop_index {
            return Ok(vec![]);
        }

        queue.push(start_index);

        while !queue.is_empty() {
            let current_index = queue.pop().unwrap();
            visited.push(current_index);
            if current_index == stop_index {
                break;
            }

            if let Some(current) = self.road_arena.get(*current_index) {
                println!("Current Node: {:?}-{}", current.node_type, current.supply);

                for neighbor_node_index in current.neighbors.iter() {
                    if neighbor_node_index == stop_index {
                        return Ok(path);
                    }

                    println!("Neighbor Node: {} | {:?}", visited.contains(&neighbor_node_index), visited);
                    if !visited.contains(&neighbor_node_index) {
                        path.push(neighbor_node_index);
                        queue.push(neighbor_node_index);
                    }
                }
            }
        }
        Err(5)
    }

    pub fn path_to_node(&self, start: Index, end: Index) -> Result<Vec<&Index>, String> {
        if self.road_arena.get(start).is_some() {
            if self.road_arena.get(end).is_some() {
                // Start with drunkards walk
                match self.get_dunkards_path(&start, &end) {
                    Ok(path) => return Ok(path),
                    Err(message) => {
                        return Err(message.to_string())
                    }
                }
            } else {
                Err(String::from("End node not found"))
            }
        } else {
            Err(String::from("Start node not found"))
        }
    }
}

pub fn main() {
    let mut network = RoadNetwork::new();
    let house1 = network.add_node(String::from("house1"), RoadNodeType::RESIDENTIAL, 10);
    let house2 = network.add_node(String::from("house2"), RoadNodeType::RESIDENTIAL, 20);
    let house3 = network.add_node(String::from("house3"), RoadNodeType::RESIDENTIAL, 30);
    // Wasting space with clones as a test for now
    let house4 = network.add_node_with_neighbors(String::from("house4"), RoadNodeType::RESIDENTIAL, 5, vec![house1.clone(), house2.clone()]);
    let business = network.add_node_with_neighbors(String::from("business"), RoadNodeType::COMMERCIAL, 100, vec![house3.clone(), house4.clone()]);
    let industry = network.add_node_with_neighbors(String::from("industry"), RoadNodeType::INDUSTRIAL, 0, vec![business.clone()]);

    match network.path_to_node(industry, house1) {
        Ok(path) => {
            println!("Path found");
            path.iter().for_each(|path_node| {
                println!("{:?}", network.road_arena.get(**path_node).unwrap());
            });
            println!("Finished");
        }
        Err(msg) => println!("{}", msg)
    };
}