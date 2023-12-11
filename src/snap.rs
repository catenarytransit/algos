use std::collections::HashMap;
use chrono::prelude::*;
use gtfs_structures;
use serde::{Serialize, Deserialize};
use tokio_postgres::{NoTls, Client};
use tokio;
#[derive(Serialize, Deserialize, Debug)]
struct GTFSGraph {
    onestop_id: String,
    old_services: Vec<String>,
    route_names: HashMap<String, String>,
    stop_names: HashMap<String, String>,
    //<route id, <stop id, <service id, Vec<stop times>>>>
    routes: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
}

impl GTFSGraph {
    fn new(onestop_id: &str) -> Self {
        Self {
            onestop_id: onestop_id.to_string(),
            old_services: Vec::new(),
            routes: HashMap::new(),
            route_names: HashMap::new(),
            stop_names: HashMap::new(),
        }
    }
    async fn to_sql(&mut self, client: &Client) {
        for (route, stops) in &self.routes {
            // Iterate over the middle HashMap
            for (stop, services) in stops {
                // Iterate over the innermost HashMap
                for (service, times) in services {
                    let json_value = serde_json::to_string(&times);
                    // Prepare the SQL statement with parameterized query
                    let statement = "INSERT INTO timetable(id, time) VALUES ($1, $2) RETURNING *";

                    // Perform the SQL INSERT query
                    let rows = client.query_one(statement, &[&format!("{}-{}-{}-{}", self.onestop_id, &route, &stop, &service), &json_value.unwrap()]).await;
                    
                    println!("{:#?}", rows.unwrap());

                }
            }
        }
    }
    async fn from_sql(&mut self, client: &Client) {

    }
    //adding connected edges
    fn add_route(&mut self, id: String, name: String) {
        let stops = HashMap::new();
        self.routes.insert(id.clone(), stops);
        self.route_names.insert(id, name);
    }
    fn exclude_service(&mut self, id: String) {
        self.old_services.push(id);
    }
    fn add_stop(&mut self, id: String, name: String) {
        self.stop_names.insert(id, name);
    }
    fn add_stoptime(&mut self, id: String, stop_id: String, service_id: String, arrival_time: u32) {//, start_date: &String, end_date: &String) {
        if self.old_services.contains(&service_id) {
            return;
        }
        if !self.routes.contains_key(&id) {
            self.add_route(id.clone(), "Kyler's Transit Line".to_string());
        }
        let mut arrival_string = (arrival_time/3600).to_string();
        if arrival_string.len() == 1 {
            arrival_string.insert_str(0, "0");
        }
        if ((arrival_time % 3600)/60).to_string().as_str().len() == 1 {
            arrival_string.push_str(format!(":0{}", ((arrival_time % 3600)/60).to_string()).as_str());
        } else {
            arrival_string.push_str(format!(":{}", ((arrival_time % 3600)/60).to_string()).as_str());
        }
        if !self.routes.get_mut(&id).unwrap().contains_key(&stop_id) {
            self.routes.get_mut(&id).unwrap().insert(stop_id.clone(), HashMap::new());
        }
        if !self.routes.get_mut(&id).unwrap().get_mut(&stop_id).unwrap().contains_key(&service_id) {
            let new_stop_times = vec![arrival_string];
            self.routes.get_mut(&id).unwrap().get_mut(&stop_id).unwrap().insert(service_id, new_stop_times);
        } else {
            self.routes.get_mut(&id).unwrap().get_mut(&stop_id).unwrap().get_mut(&service_id).unwrap().push(arrival_string);
        }
    }
    fn clean(&mut self) {
        for route in &mut self.routes {
            for stop in route.1 {
                for service in stop.1 {
                    service.1.sort();
                }
            }
        }
    }
}
#[tokio::main]
async fn main() {
    let gfts_rail = gtfs_structures::Gtfs::new("gtfs_rail.zip").unwrap();
    let mut graph: GTFSGraph = GTFSGraph::new("f-9q5-metro~losangeles~rail"); 
    for route in gfts_rail.routes {
        graph.add_route(route.1.id, route.1.long_name);
    }
    let local: DateTime<Local> = Local::now();
    let formatted_date = local.format("%Y-%m-%d").to_string();
    //let mut future_services: Vec<String> = Vec::new();
    //let mut services: Vec<String> = Vec::new();
    for service in gfts_rail.calendar {
        if service.1.end_date.to_string() <= formatted_date {
            /*if service.1.start_date.to_string() >= formatted_date {
                graph.exclude_service(service.1.id.clone());
            } else {
                graph.exclude_service(service.1.id.clone());
            }*/
            graph.exclude_service(service.1.id.clone());
        }
        //eprintln!("{} {} {} {} {} ", formatted_date, formatted_date <= service.1.start_date.to_string(), service.1.start_date.to_string(), formatted_date <= service.1.end_date.to_string(), service.1.end_date.to_string());
    }
    //eprintln!("{:#?}", graph.old_services);
    for trip in gfts_rail.trips {
        //println!("{}", trip.1.id);
        for stop_times in trip.1.stop_times {
            if !graph.stop_names.contains_key(&stop_times.stop.id) {
                graph.add_stop(stop_times.stop.id.clone(), stop_times.stop.name.clone())
            }
            graph.add_stoptime(trip.1.route_id.clone(), stop_times.stop.id.clone(), trip.1.service_id.clone(), stop_times.arrival_time.unwrap());
        }
    }
    graph.clean();
    //println!("{:#?}", graph);
    let conn_string = "postgresql://lolpro11:lolpro11@localhost/catenary";

    // Establish a connection to the database
    let (client, connection) = tokio_postgres::connect(conn_string, NoTls).await.unwrap();
    tokio::spawn(connection);
    let x = client.execute(
        "CREATE TABLE IF NOT EXISTS timetable (
            id VARCHAR PRIMARY KEY,
            time VARCHAR
        );",
        &[],
    ).await;
    println!("{:#?}", x);
    graph.to_sql(&client).await;
    println!("{:#?}", x);
    //let retrieved_data = query_data(&client).await;
    //println!("Retrieved data: {:#?}", retrieved_data);
}