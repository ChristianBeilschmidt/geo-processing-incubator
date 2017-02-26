use chrono::{DateTime, UTC};
use geo::{/*LineString, MultiLineString, */MultiPoint,/* MultiPolygon,*/ Point, /*Polygon*/};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice;
use std::string::String;
use std::iter::{Iterator};
use vectors::RangeIteratorAdapter;
use std::mem;

#[derive(Clone)]
pub struct TimeInterval {
    start: Option<DateTime<UTC>>,
    end: Option<DateTime<UTC>>,
}

impl TimeInterval {
    pub fn start(&self) -> &Option<DateTime<UTC>> {
        &self.start
    }

    pub fn end(&self) -> &Option<DateTime<UTC>> {
        &self.end
    }
}

pub struct FeatureCollection<T> {
    // bind type
    phantom: PhantomData<T>,

    // features
    start_feature: Vec<usize>, // the last index points to invalid range

    // geom
    points: Vec<Point<f64>>,

    // time
    time: Vec<TimeInterval>,

    // data
    global_text: HashMap<String, String>,
    global_numeric: HashMap<String, f64>,
    text: HashMap<String, Vec<String>>,
    numeric: HashMap<String, Vec<f64>>,
}

pub struct GeomIter<'c, T: 'c> {
    feature_collection: &'c FeatureCollection<T>,
    range_iter: RangeIteratorAdapter<slice::Iter<'c, usize>>,
}

impl<'c, T: 'c> GeomIter<'c, T> {
    fn new(feature_collection: &'c FeatureCollection<T>) -> Self {
        GeomIter {
            feature_collection: feature_collection,
            range_iter: feature_collection.feature_indizes_iter(),
        }
    }
}

impl<'c> Iterator for GeomIter<'c, Point<f64>> {
    type Item = Point<f64>;
    fn next(&mut self) -> Option<Self::Item> {
        self.range_iter.next()
                       .and_then(|range| self.feature_collection.get_geom(range.start))
    }
}

impl<'c> Iterator for GeomIter<'c, MultiPoint<f64>> {
    type Item = MultiPoint<f64>;
    fn next(&mut self) -> Option<Self::Item> {
        self.range_iter.next()
                       .and_then(|range| {
                           let points = range.flat_map(
                               |index| self.feature_collection.points.get(index)
                                                                     .cloned()
                                                                     .into_iter()
                           ).collect();
                           Some(MultiPoint(points))
                       })
    }
}

pub trait FeatureCollectionGeom<T> {
    fn get_geom(&self, index: usize) -> Option<T>;

    fn geom_iter(&self) -> GeomIter<T>;
}

impl FeatureCollectionGeom<Point<f64>> for FeatureCollection<Point<f64>> {
    fn get_geom(&self, index: usize) -> Option<Point<f64>> {
        self.points.get(index).cloned()
    }

    fn geom_iter(&self) -> GeomIter<Point<f64>> {
        GeomIter::new(self)
    }
}

impl FeatureCollectionGeom<MultiPoint<f64>> for FeatureCollection<MultiPoint<f64>> {
    fn get_geom(&self, index: usize) -> Option<MultiPoint<f64>> {
        match (self.start_feature.get(index), self.start_feature.get(index+1)) {
            (Some(start), Some(end)) => {
                let points = (*start..*end).flat_map(
                    |index| self.points.get(index).cloned().into_iter()
                ).collect();
                Some(MultiPoint(points))
            },
            _ => None
        }
    }

    fn geom_iter(&self) -> GeomIter<MultiPoint<f64>> {
        GeomIter::new(self)
    }
}

impl<T> FeatureCollection<T> {
    // pub fn feature_indizes_iter<'s>(&'s self) -> impl Iterator<Item=Range<usize>> + 's
    // pub fn feature_indizes_iter(&self) -> Map<Zip<slice::Iter<usize>, Skip<slice::Iter<usize>>>, fn((&usize, &usize)) -> Range<usize>>
    // {
    //     fn range_creator((start, end): (&usize, &usize)) -> Range<usize> {
    //         *start..*end
    //     }
    //     self.start_feature.iter()
    //                       .zip(self.start_feature.iter().skip(1))
    //                       .map(range_creator)
    // }

    pub fn get_feature_count(&self) -> usize {
        self.start_feature.len() - 1
    }

    pub fn feature_indizes_iter(&self) -> RangeIteratorAdapter<slice::Iter<usize>> {
        RangeIteratorAdapter::new(self.start_feature.iter())
    }

    pub fn get_time(&self, index: usize) -> Option<&TimeInterval> {
        self.time.get(index)
    }

    pub fn set_time(&mut self, index: usize, time: TimeInterval) -> Result<(), ()> {
        match self.time.get_mut(index) {
            Some(time_interval) => {
                *time_interval = time;
                Ok(())
            },
            _ => Err(())
        }
    }

    pub fn has_time(&self) -> bool {
        self.time.len() == self.start_feature.len() - 1
    }

    pub fn get_global_text(&self, key: &str) -> Option<&str> {
        self.global_text.get(key)
            .map(String::as_ref)
    }

    pub fn get_global_numeric(&self, key: &str) -> Option<&f64> {
        self.global_numeric.get(key)
    }

    pub fn get_text(&self, index: usize, key: &str) -> Option<&str> {
        self.text.get(key)
            .and_then(|text_vector| text_vector.get(index))
            .map(String::as_ref)
    }

    pub fn get_numeric(&self, index: usize, key: &str) -> Option<&f64> {
        self.numeric.get(key)
            .and_then(|numeric_vector| numeric_vector.get(index))
    }

    pub fn filter_inplace(&mut self, to_keep: &[bool]) {
        if to_keep.len() != self.get_feature_count() {
            panic!("The lengths must match.");
        }

        let keep_count = to_keep.iter().map(|&keep| if keep {1} else {0}).sum();

        if keep_count == self.get_feature_count() {
            return; // nothing to do
        }

        let time = mem::replace(&mut self.time, Vec::new());
        self.time = time.into_iter()
                        .zip(to_keep.iter())
                        .filter_map(|(time, &keep)| if keep {Some(time)} else {None})
                        .collect();

        for value in &mut self.text.values_mut() {
            let filtered_value = mem::replace(value, Vec::new());
            *value = filtered_value.into_iter()
                         .zip(to_keep.iter())
                         .filter_map(|(value, &keep)| if keep {Some(value)} else {None})
                         .collect();
        }

        for value in &mut self.numeric.values_mut() {
            let filtered_value = mem::replace(value, Vec::new());
            *value = filtered_value.into_iter()
                         .zip(to_keep.iter())
                         .filter_map(|(value, &keep)| if keep {Some(value)} else {None})
                         .collect();
        }

        let mut start_feature = Vec::with_capacity(keep_count);
        let mut points = Vec::with_capacity(keep_count);

        for feature_range in self.feature_indizes_iter().zip(to_keep)
                                                        .filter_map(|(range, &keep)| {
                                                            if keep {
                                                                Some(range)
                                                            } else {
                                                                None
                                                            }
                                                        })
        {
            let new_point_index = start_feature.len();
            start_feature.push(new_point_index);
            points.extend_from_slice(&self.points[feature_range]);
        }
        {
            let new_point_index = start_feature.len();
            start_feature.push(new_point_index); // pointer to end + 1
        }

        self.start_feature = start_feature;
        self.points = points;
    }
}

// pub struct PointIter<'c> {
//     feature_collection: &'c FeatureCollection<Point<f64>>,
//     range_iter: RangeIteratorAdapter<slice::Iter<'c, usize>>,
// }
//
// impl<'c> PointIter<'c> {
//     fn new(feature_collection: &'c FeatureCollection<Point<f64>>) -> Self {
//         PointIter {
//             feature_collection: feature_collection,
//             range_iter: RangeIteratorAdapter::new(feature_collection.start_feature.iter()),
//         }
//     }
// }
//
// impl<'c> Iterator for PointIter<'c> {
//     type Item = Point<f64>;
//     fn next(&mut self) -> Option<Point<f64>> {
//         self.range_iter.next()
//                        .and_then(|range| self.feature_collection.get_geom(*range.start))
//     }
// }

impl FeatureCollection<Point<f64>> {
    pub fn new(points: &[Point<f64>], time: Option<&[TimeInterval]>) -> Result<FeatureCollection<Point<f64>>, ()> {
        if let Some(time_intervals) = time {
            if time_intervals.len() != points.len() {
                return Err(()); // TODO: meaningful error
            }
        }

        Ok(
            FeatureCollection::<Point<f64>> {
                phantom: PhantomData,
                start_feature: (0..points.len()+1).collect(),

                // geom
                points: points.to_vec(),

                // time
                time: time.map(|s| s.to_vec()).unwrap_or_default(),

                // data
                global_text: HashMap::new(),
                global_numeric: HashMap::new(),
                text: HashMap::new(),
                numeric: HashMap::new(),
            }
        )
    }
}
