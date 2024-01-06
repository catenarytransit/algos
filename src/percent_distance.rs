
use geographiclib_rs::{Geodesic, DirectGeodesic, InverseGeodesic};
use geo::LineString;
use itertools::Itertools;
// given two points (A, B) and a proportion P,
// find the point X along the geodesic P of the length of the geodesic from A
fn percent_geodesic(a: (f64, f64), b: (f64, f64), p: f64) -> (f64, f64) {
    let geod = Geodesic::wgs84();
    let (s_ab, azi_a, _, _) = geod.inverse(a.0, a.1, b.0, b.1);
    geod.direct(a.0, a.1, azi_a, p * s_ab)
}

fn percent_linestring(l: LineString, p: f64) -> (f64, f64) {
    let geod = Geodesic::wgs84();
    let mut distances: Vec<f64> = Vec::new();
    for (a, b) in l.0.iter().tuple_windows() {
        distances.push(geod.inverse(a.x, a.y, b.x, b.y));
    }
    let distance: f64 = p * distances.iter().sum::<f64>();
    let mut sum = 0.0;
    let mut i = 0;
    for _ in 0..distances.len() {
        sum += distances[i];
        if sum > distance {
            break;
        }
        i += 1;
    }
    let (azi_a, _, _) = geod.inverse(l[i].x, l[i].y, l[i + 1].x, l[i + 1].y);
    geod.direct(l[i].x, l[i].y, azi_a, distance - sum + distances[i])
}

fn main() {
    println!("24 km case:");
    println!("{:?}", percent_geodesic((52.0, 5.0), (51.4, 6.0), 0.25));
    println!("1000 km case:");
    println!("{:?}", percent_geodesic((42.0, 29.0), (39.0, -77.0), 0.5));
    println!("12200 km case:");
    println!("{:?}", percent_geodesic((42.0, 29.0), (-35.0, -70.0), 0.75));
}