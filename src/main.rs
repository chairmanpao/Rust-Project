use csv;
use rand::{Rng};
use std::{collections::HashMap, fs::File};

#[derive(Debug, Clone, PartialEq)]
struct DataPoint {
    lat: f64,
    lon: f64,
    weapon: String,
    race: String,
}

impl DataPoint {
    fn new(lat: &str, lon: &str, weapon: &str, race: &str) -> Self {
        Self {
            lat: lat.parse().unwrap(),
            lon: lon.parse().unwrap(),
            weapon: weapon.to_string(),
            race: race.to_string(),
        }
    }
    fn distance(&self, other: &Self) -> f64 {
        ((self.lon - other.lon).powi(2) + (self.lat - other.lat).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DataObject {
    weapon: String,
    race: String,
}

impl DataObject {
    fn from(dp: &DataPoint) -> Self {
        Self {
            weapon: dp.weapon.clone(),
            race: dp.race.clone(),
        }
    }
}

struct WeightedGraph {
    graph: Vec<Vec<f64>>,
    nodes: Vec<DataPoint>,
}

impl WeightedGraph {
    fn new(nodes: &Vec<DataPoint>) -> Self {
        let num_nodes = nodes.len();
        let mut new_graph = Self {
            graph: Vec::new(),
            nodes: Vec::new(),
        };
        for _ in 0..num_nodes {
            let mut new_vec = Vec::new();
            new_vec.resize(num_nodes, 0.0_f64);
            new_graph.graph.push(new_vec);
        }

        for (i, node) in nodes.iter().enumerate() {
            new_graph.nodes.push(node.clone());
            for j in 0..i {
                let distance = node.distance(&new_graph.nodes[j]);
                new_graph.graph[i][j] = distance;
                new_graph.graph[j][i] = distance;
            }
        }
        new_graph
    }
}

fn main() {
    let file =
        File::open("C:\\Users\\ChairmanPao\\Desktop\\fatal-police-shootings-data.csv").unwrap();
    let mut reader = csv::Reader::from_reader(file);
    let mut records = Vec::new();
    for record in reader.records().flatten() {
        if record[8].len() == 0 || record[9].len() == 0 || record[4].len() == 0 || record[14].len() == 0{
            continue;
        }
        let dp = DataPoint::new(&record[8], &record[9], &record[4], &record[14]);
        records.push(dp);
    }
    let graph = WeightedGraph::new(&records);
    let clusters = kmeans_cluster(6, graph);
    for cluster in clusters {
        let data_objs: Vec<DataObject> = cluster.iter().map(|dp| {DataObject::from(dp)}).collect();
        let mut race_map: HashMap<String, usize> = HashMap::new();
        let mut weapon_map: HashMap<String, usize> = HashMap::new();
        for obj in data_objs {
            if let Some(v) = race_map.get_mut(&obj.race) {
                *v += 1;
            } else {
                race_map.insert(obj.race, 1);
            }
            if let Some(v) = weapon_map.get_mut(&obj.weapon) {
                *v += 1;
            } else {
                weapon_map.insert(obj.weapon, 1);
            }
        }
        let mut max_weapon: String = String::new();
        let mut max_weapon_cnt: usize = 0;
        let mut max_race: String = String::new();
        let mut max_race_cnt: usize = 0;
        for (key, val) in weapon_map.iter() {
            if *val > max_weapon_cnt {
                max_weapon = key.clone();
                max_weapon_cnt = *val;
            }
        }
        for (key, val) in race_map.iter() {
            if *val > max_race_cnt {
                max_race = key.clone();
                max_race_cnt = *val;
            }
        }
        println!("Maxes: weapon: {max_weapon}:{max_weapon_cnt}, race: {max_race}:{max_race_cnt}")
    }
}

fn kmeans_cluster(k: usize, graph: WeightedGraph) -> Vec<Vec<DataPoint>> {
    let mut k_nodes = Vec::new();
    let mut old_k_nodes = Vec::new();
    let mut rng = rand::thread_rng();
    while k_nodes.len() < k {
        let random_num = rng.gen::<usize>()%graph.nodes.len();
        let p: &DataPoint = &graph.nodes[random_num];
        if !k_nodes.contains(p) {
            k_nodes.push(p.clone());
        }
    }
    loop {
        let mut clusters: HashMap<usize, Vec<DataPoint>> = HashMap::new();
        for i in 0..k {
            clusters.insert(i, Vec::new());
        }
        for node in &graph.nodes {
            let mut least_i = usize::MAX;
            let mut least_distance = f64::MAX;
            for i in 0..k {
                let dist = node.distance(&k_nodes[i]);
                if dist < least_distance {
                    least_distance = dist;
                    least_i = i;
                }
            }
            clusters.get_mut(&least_i).unwrap().push(node.clone());
        }
        old_k_nodes.clear();
        old_k_nodes.append(&mut k_nodes);
        for i in 0..k {
            k_nodes.push(calculate_average_datapoint(clusters.get(&i).unwrap()));
        }
        let mut found = false;
        for i in 0..k{
            if old_k_nodes[i].distance(&k_nodes[i]) != 0.0_f64{
                found = true;
            }
        }
        if !found{
            let mut result = Vec::new();
            for value in clusters.values(){
                result.push(value.clone());
            }
            return result;
        }
    }
}

fn calculate_average_datapoint(points: &Vec<DataPoint>) -> DataPoint {
    let mut lat = 0.0_f64;
    let mut lon = 0.0_f64;
    for point in points {
        lat += point.lat;
        lon += point.lon;
    }
    lat /= points.len() as f64;
    lon /= points.len() as f64;
    DataPoint {
        lat,
        lon,
        weapon: String::new(),
        race: String::new(),
    }
}

#[test]
fn test_calculate_average_datapoint() {
    let point = calculate_average_datapoint(&vec![DataPoint::new("1.0", "1.0", "", ""), DataPoint::new("0.0", "0.0", "", "")]);
    assert!(point.lat == 0.5);
    assert!(point.lon == 0.5);
}

/*1. Select n=k random nodes
2. Cluster all other nodes based on distance to those 4 nodes
3. For each cluster, calculate the mean point of each cluster
4. Perform clustering again, using mean point for each cluster as the new clustering central point
5. Repeat from 2 until the delta(mean) is zero/low*/
