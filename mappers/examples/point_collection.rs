extern crate geo;
extern crate mappers;

use mappers::vectors::{FeatureCollection, FeatureCollectionGeom, OgrSource};
use geo::Point;

fn main() {
    let source = OgrSource::new("examples/data/points.geojson");
    let feature_collection: FeatureCollection<Point<f64>> = source.into_point_collection()
                                                                  .expect("Something went wrong.");

    for point in feature_collection.geom_iter() {
        println!("{:?}", point);
    }
}
