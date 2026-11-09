//! The OpenOrienteering Mapper reader and map model come from `freemapper-omap` (freemapper-core);
//! this module keeps the old paths (`crate::omap::read::read_map_file` …) working.

pub use freemapper_omap::*;

#[cfg(test)]
mod tests {
    #[test]
    fn omap_comes_from_core() {
        assert!(std::any::type_name::<crate::omap::MapDocument>().starts_with("freemapper_omap::"));
    }
}
