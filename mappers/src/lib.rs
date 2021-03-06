extern crate iron;
extern crate params;
extern crate router;
extern crate image;
extern crate gdal;
extern crate colorizers;
extern crate num;
extern crate chrono;

#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

pub mod mappers_handler;
//pub mod raster_traits;
//pub mod spatial_reference;
pub mod gdal_source;
pub mod errors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
