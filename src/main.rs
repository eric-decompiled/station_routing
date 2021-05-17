use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Need path of input file as only argument");
        return;
    }
    let filename = &args[1];
    let input = fs::read_to_string(filename).expect("Unable to read input file");
    let routes = RouteMap::new(&input);

    for (index, cmd) in COMMANDS.iter().enumerate() {
        println!("Output #{}: {}", index + 1, routes.eval_cmd(cmd));
    }
}

#[derive(Debug)]
enum Command {
    ABCDistance,
    ADDistance,
    ADCDistance,
    AEBCDDistance,
    AEDDistance,
    CCircular,
    ACFourStop,
    ACExpress,
    BCircle,
    CLessThanCircular,
}

static COMMANDS: [Command; 10] = [
    Command::ABCDistance,
    Command::ADDistance,
    Command::ADCDistance,
    Command::AEBCDDistance,
    Command::AEDDistance,
    Command::CCircular,
    Command::ACFourStop,
    Command::ACExpress,
    Command::BCircle,
    Command::CLessThanCircular,
];

#[derive(Debug)]
struct Route {
    distance: u32,
    stops: Vec<String>,
}
type Stations = HashMap<String, HashMap<String, u32>>;
#[derive(Debug)]
struct RouteMap {
    graph: Stations,
}

impl Route {
    fn current_station(&self) -> &str {
        self.stops
            .last()
            .expect("current_station called on empty route")
    }
}

/// Input as list in form of `XYD, ` where X = starting point, Y = destination, D = distance
/// Places as single a-Z char, distance as digits
impl RouteMap {
    pub fn new(input: &str) -> Self {
        let pattern = Regex::new(r"([a-zA-Z])([a-zA-Z])(\d+)").expect("invalid input regex");
        let mut graph: Stations = HashMap::new();
        for route in pattern.captures_iter(input) {
            let start = &route[1];
            let destination = &route[2];
            let distance: u32 = route[3].parse().expect("Expecting u32 as distance");
            if !graph.contains_key(start) {
                graph.insert(start.to_owned(), HashMap::new());
            };
            let station = graph.get_mut(start).expect("Station was not present");
            station.insert(destination.to_owned(), distance);
        }
        Self { graph }
    }

    pub fn eval_cmd(&self, cmd: &Command) -> String {
        let result = match cmd {
            Command::ABCDistance => self.route_distance(&["A", "B", "C"]),
            Command::ADDistance => self.route_distance(&["A", "D"]),
            Command::ADCDistance => self.route_distance(&["A", "D", "C"]),
            Command::AEBCDDistance => self.route_distance(&["A", "E", "B", "C", "D"]),
            Command::AEDDistance => self.route_distance(&["A", "E", "D"]),
            Command::CCircular => self.circular_route("C", 3),
            Command::ACFourStop => self.exact_stops("A", "B", 4),
            Command::ACExpress => self.shortest_route("A", "C"),
            Command::BCircle => self.shortest_route("B", "B"),
            Command::CLessThanCircular => self.routes_less_than("C", "C", 30),
        };
        match result {
            Some(num) => num.to_string(),
            None => "NO SUCH ROUTE".to_string(),
        }
    }

    /// Takes a list of stops and will return the distance travelled. Returns None if route is not possible
    fn route_distance(&self, stops: &[&str]) -> Option<u32> {
        let mut distance = 0;
        let mut current_station = "";
        for stop in stops {
            if current_station != "" {
                let station = self.graph.get(current_station)?;
                distance += station.get(stop.to_owned())?;
            }
            current_station = stop;
        }
        Some(distance)
    }

    /// Finds the number of routes that begin and end at a station less than a number of stops
    fn circular_route(&self, start: &str, max_stops: u32) -> Option<u32> {
        let mut stops_made = 0;
        let mut next_stops: Vec<&str> = vec![];
        let mut stops: Vec<&str> = vec![start];
        let mut count = 0;
        while stops_made < max_stops {
            for stop in &stops {
                let station = self.graph.get(stop.to_owned())?;
                for (destination, _distance) in station {
                    if destination == start {
                        count += 1
                    };
                    next_stops.push(destination);
                }
            }
            stops = next_stops;
            next_stops = vec![];
            stops_made += 1;
        }
        Some(count)
    }

    // Finds the number of routes between two places with an exact number of stops
    fn exact_stops(&self, start: &str, destination: &str, target_stops: u32) -> Option<u32> {
        let mut stops_made = 0;
        let start = Route {
            stops: vec![start.to_owned()],
            distance: 0,
        };
        let mut stops = self.next_stops(start);
        let mut next_stops = vec![];
        while stops_made < target_stops {
            for route in stops {
                next_stops.extend(self.next_stops(route))
            }
            stops = next_stops;
            next_stops = vec![];
            stops_made += 1;
        }
        let mut count = 0;
        for stop in stops {
            if stop.current_station() == destination {
                count += 1;
            };
        }
        Some(count)
    }

    /// Finds the distance of the shortest route between two places
    fn shortest_route(&self, start: &str, destination: &str) -> Option<u32> {
        let mut shortest_route = Route {
            stops: vec![],
            distance: std::u32::MAX,
        };
        let start = Route {
            stops: vec![start.to_owned()],
            distance: 0,
        };
        let mut stops = self.next_stops(start);
        let mut next_stops = vec![];
        while stops.len() > 0 {
            for route in stops {
                if route.distance < shortest_route.distance {
                    if route.current_station() == destination {
                        shortest_route = route;
                    } else {
                        next_stops.extend(self.next_stops(route));
                    }
                }
            }
            stops = next_stops;
            next_stops = vec![];
        }
        if shortest_route.distance == std::u32::MAX {
            None
        } else {
            Some(shortest_route.distance)
        }
    }

    /// Finds the number of routes between two places less than a certain distance
    fn routes_less_than(&self, start: &str, destination: &str, max_distance: u32) -> Option<u32> {
        let mut count = 0;
        let start = Route {
            stops: vec![start.to_owned()],
            distance: 0,
        };
        let mut stops = self.next_stops(start);
        let mut next_stops = vec![];
        while stops.len() > 0 {
            for route in stops {
                if route.distance < max_distance {
                    if route.current_station() == destination {
                        count += 1
                    }
                    next_stops.extend(self.next_stops(route));
                }
            }
            stops = next_stops;
            next_stops = vec![];
        }
        Some(count)
    }

    /// Takes a route and returns a Vec of routes representing all possible direct routes.
    fn next_stops(&self, route: Route) -> Vec<Route> {
        let mut new_routes: Vec<Route> = vec![];
        let station = self.graph.get(route.current_station()).unwrap();
        for (destination, distance) in station {
            let mut r = Route {
                stops: route.stops.to_owned(),
                distance: route.distance,
            };
            r.distance += distance;
            r.stops.push(destination.to_string());
            new_routes.push(r);
        }
        new_routes
    }
}

#[test]
fn check_distance() {
    let routes = RouteMap::new("AB2, BC3");
    let distance = routes.route_distance(&["A", "B", "C"]).unwrap();
    assert_eq!(5, distance);
}
#[test]
fn check_no_route() {
    let routes = RouteMap::new("AB3, CD4");
    let distance = routes.route_distance(&["A", "C"]);
    assert_eq!(None, distance)
}

#[test]
fn check_circular() {
    let routes = RouteMap::new("CD3, DE3, EC3, EB4, BC4");
    let count = routes.circular_route("C", 4).unwrap();
    assert_eq!(2, count)
}

#[test]
fn check_exact_stops() {
    let routes = RouteMap::new("AB3, AC2, BC3, BA2, CA7");
    println!("{:?}", routes);
    let count = routes.exact_stops("A", "B", 2).unwrap();
    assert_eq!(2, count)
}

#[test]
fn check_shortest_route() {
    let routes = RouteMap::new("AB1, BC1, AD3, DC3");
    let distance = routes.shortest_route("A", "C").unwrap();
    assert_eq!(2, distance)
}

#[test]
fn check_routes_less_than() {
    let routes = RouteMap::new("AB1, BC1, BA2, CA3, CD5, DA5");
    let count = routes.routes_less_than("A", "A", 8).unwrap();
    assert_eq!(3, count)
}

#[test]
fn check_all_commands() {
    let routes = RouteMap::new("AB5, BC4, CD8, DC8, DE6, AD5, CE2, EB3, AE7");
    let expected = [
        "9",
        "5",
        "13",
        "22",
        "NO SUCH ROUTE",
        "2",
        "3",
        "9",
        "9",
        "7",
    ];
    for (expected, cmd) in expected.iter().zip(COMMANDS.iter()) {
        let result = routes.eval_cmd(cmd);
        assert_eq!(expected, &result)
    }
}
