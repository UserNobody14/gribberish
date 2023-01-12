use mappers::{projections::LambertConformalConic, Ellipsoid, Projection};

use crate::{
    templates::template::{Template, TemplateType},
    utils::{bit_array_from_bytes, read_u32_from_bytes},
};

use super::{
    tables::{
        EarthShape, ProjectionCenter, ProjectionCenterFlags, ScanningMode, ScanningModeFlags,
    },
    GridDefinitionTemplate,
};

pub struct LambertConformalTemplate {
    data: Vec<u8>,
}

impl Template for LambertConformalTemplate {
    fn template_type(&self) -> TemplateType {
        TemplateType::Grid
    }

    fn template_number(&self) -> u16 {
        0
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_name(&self) -> &str {
        "Lambert Confromal"
    }
}

impl LambertConformalTemplate {
    pub fn new(data: Vec<u8>) -> Self {
        LambertConformalTemplate { data }
    }

    pub fn earth_shape(&self) -> EarthShape {
        self.data[14].into()
    }

    pub fn earth_radius_scale_factor(&self) -> u8 {
        self.data[15]
    }

    pub fn earth_radius_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 16).unwrap_or(0)
    }

    pub fn earth_major_axis_scale_factor(&self) -> u8 {
        self.data[20]
    }

    pub fn earth_major_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 21).unwrap_or(0)
    }

    pub fn earth_minor_axis_scale_factor(&self) -> u8 {
        self.data[25]
    }

    pub fn earth_minor_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 26).unwrap_or(0)
    }

    pub fn number_of_points_on_x_axis(&self) -> u32 {
        read_u32_from_bytes(&self.data, 30).unwrap_or(0)
    }

    pub fn number_of_points_on_y_axis(&self) -> u32 {
        read_u32_from_bytes(&self.data, 34).unwrap_or(0)
    }

    pub fn latitude_of_first_grid_point(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 38).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_first_grid_point(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 42).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> Vec<u8> {
        bit_array_from_bytes(&self.data[46..47])
    }

    pub fn latitude_of_dx_dy(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 47).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_paralell_meridian_to_latitude_increase(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 51).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn x_direction_grid_length(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 55).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-3.0))
    }

    pub fn y_direction_grid_length(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 59).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-3.0))
    }

    pub fn projection_centre_flags(&self) -> ProjectionCenterFlags {
        ProjectionCenter::read_flags(self.data[63])
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[64])
    }

    pub fn latin_1(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 65).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn latin_2(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 69).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn latitude_of_southern_pole(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 73).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_southern_pole(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 77).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn projection(&self) -> Result<LambertConformalConic, String> {
        let mut lng = self.longitude_of_first_grid_point(); 
        println!("{lng}");
        lng = if lng > 180.0 {
            lng - 360.0
        } else {
            lng
        };

        LambertConformalConic::new(
            lng,
            self.latitude_of_first_grid_point(),
            self.latin_1(),
            self.latin_2(),
            Ellipsoid::wgs84(),
        )
        .map_err(|e| format!("Failed to create lambert conformal conic projection: {e}"))
    }

    pub fn project_axes(&self) -> Result<(LambertConformalConic, Vec<f64>, Vec<f64>), String> {
        let mut start_lng = self.longitude_of_first_grid_point(); 
        start_lng = if start_lng > 180.0 {
            start_lng - 360.0
        } else {
            start_lng
        };

        let projection = self.projection()?;
        let (start_x, start_y) = projection
            .project(
                start_lng,
                self.latitude_of_first_grid_point(),
            )
            .map_err(|e| {
                format!(
                    "Failed to project start coordinates to lambert conformal conic coords: {e}"
                )
            })?;

        let dx = if self.scanning_mode_flags()[0] == ScanningMode::PlusI {self.x_direction_grid_length() } else {-self.x_direction_grid_length()};
        let dy = if self.scanning_mode_flags()[1] == ScanningMode::PlusJ {self.y_direction_grid_length() } else {-self.y_direction_grid_length()};

        let x = (0..self.number_of_points_on_x_axis())
            .map(|i| start_x + dx * i as f64)
            .collect();

        let y = (0..self.number_of_points_on_y_axis())
            .map(|i| start_y + dy * i as f64)
            .collect();

        Ok((projection, x, y))
    }
}

impl GridDefinitionTemplate for LambertConformalTemplate {
    fn proj_string(&self) -> String {
        format!(
            "+proj=lcc lon_0={} lat_0={} lat_1={} lat_2={}",
            self.longitude_of_paralell_meridian_to_latitude_increase(),
            self.latitude_of_dx_dy(),
            self.latin_1(),
            self.latin_2()
        )
    }

    fn crs(&self) -> String {
        // This is probably not right
        "EPSG:2154".to_string()
    }

    fn grid_point_count(&self) -> usize {
        (self.number_of_points_on_x_axis() * self.number_of_points_on_y_axis()) as usize
    }

    fn start(&self) -> (f64, f64) {
        *self
            .latlng()
            .first()
            .expect("Failed to get last lat/lng pair")
    }

    fn end(&self) -> (f64, f64) {
        *self
            .latlng()
            .last()
            .expect("Failed to get last lat/lng pair")
    }

    fn latitude_count(&self) -> usize {
        self.number_of_points_on_y_axis() as usize
    }

    fn longitude_count(&self) -> usize {
        self.number_of_points_on_x_axis() as usize
    }

    fn latlng(&self) -> Vec<(f64, f64)> {
        let (projection, x, y) = self
            .project_axes()
            .expect("Failed to project LCC coordinates");

        x.iter()
            .flat_map(|x_coord| {
                y.iter()
                    .map(|y_coord| {
                        projection
                            .inverse_project(*x_coord, *y_coord)
                            .expect("Failed to inverse project from xy to lnglat")
                    })
                    .collect::<Vec<(f64, f64)>>()
            })
            .collect::<Vec<(f64, f64)>>()
    }
}