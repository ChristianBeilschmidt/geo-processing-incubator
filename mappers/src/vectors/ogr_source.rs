use std::path::Path;
use gdal::vector;
use gdal::vector::{Dataset, Geometry};
use geo;
// use geo::ToGeo;
use vectors::FeatureCollection;

pub struct OgrSource {
    dataset: Dataset,
}

impl OgrSource {
    pub fn new(csv_file: &str) -> OgrSource {
        OgrSource {
            dataset: Dataset::open(
                Path::new(csv_file)
            ).expect("This file does not exist.")
        }
    }

    pub fn into_point_collection(mut self) -> Result<FeatureCollection<geo::Point<f64>>, ()> {
        if self.dataset.count() != 1 {
            return Err(());
        }

        let layer = self.dataset.layer(0).unwrap();

        let mut points = Vec::new();

        for feature in layer.features() {
            let geometry: &vector::Geometry = feature.geometry();
            
            // if let geo::Geometry::Point(point) = geometry.to_geo() {
            //     points.push(point)
            // }

            let (x, y, _) = geometry.get_point(0);
            points.push(geo::Point(geo::Coordinate{x: x, y: y}));
        }

        FeatureCollection::<geo::Point<f64>>::new(&points, None)
    }
}
