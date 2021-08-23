use pyo3::exceptions::{PyIOError, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;

use crate::container::section_directory;
use crate::container::section_loader;
use crate::core::error::{DwgError, ErrorKind};
use crate::dwg::file_open;
use crate::dwg::version;
use crate::entities;
use crate::objects;

#[pyfunction]
pub fn hello_from_bin() -> String {
    "Hello from ezdwg!".to_string()
}

#[pyfunction]
pub fn detect_version(path: &str) -> PyResult<String> {
    let tag = file_open::read_version_tag(path).map_err(to_py_err)?;
    let version = version::detect_version(&tag).map_err(to_py_err)?;
    Ok(version.as_str().to_string())
}

#[pyfunction]
pub fn list_section_locators(path: &str) -> PyResult<Vec<(String, u32, u32)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let directory = section_directory::parse(&bytes).map_err(to_py_err)?;
    let result = directory
        .records
        .into_iter()
        .map(|record| (record.kind().label(), record.offset, record.size))
        .collect();
    Ok(result)
}

#[pyfunction]
pub fn read_section_bytes(path: &str, index: usize) -> PyResult<Vec<u8>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let directory = section_directory::parse(&bytes).map_err(to_py_err)?;
    let section =
        section_loader::load_section_by_index(&bytes, &directory, index, &Default::default())
            .map_err(to_py_err)?;
    Ok(section.data.to_vec())
}

#[pyfunction(signature = (path, limit=None))]
pub fn list_object_map_entries(path: &str, limit: Option<usize>) -> PyResult<Vec<(u64, u32)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut entries: Vec<(u64, u32)> = index
        .objects
        .iter()
        .map(|obj| (obj.handle.0, obj.offset))
        .collect();
    if let Some(limit) = limit {
        if entries.len() > limit {
            entries.truncate(limit);
        }
    }
    Ok(entries)
}

#[pyfunction(signature = (path, limit=None))]
pub fn list_object_headers(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u32, u32, u16)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let header = objects::parse_object_header_r2000(&bytes, obj.offset).map_err(to_py_err)?;
        result.push((obj.handle.0, obj.offset, header.data_size, header.type_code));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn list_object_headers_with_type(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u32, u32, u16, String, String)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let header = objects::parse_object_header_r2000(&bytes, obj.offset).map_err(to_py_err)?;
        let type_name = objects::object_type_name(header.type_code);
        let type_class = objects::object_type_class(header.type_code)
            .as_str()
            .to_string();
        result.push((
            obj.handle.0,
            obj.offset,
            header.data_size,
            header.type_code,
            type_name,
            type_class,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, type_codes, limit=None))]
pub fn list_object_headers_by_type(
    path: &str,
    type_codes: Vec<u16>,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u32, u32, u16, String, String)>> {
    if type_codes.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let filter: std::collections::HashSet<u16> = type_codes.into_iter().collect();
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let header = objects::parse_object_header_r2000(&bytes, obj.offset).map_err(to_py_err)?;
        if !filter.contains(&header.type_code) {
            continue;
        }
        let type_name = objects::object_type_name(header.type_code);
        let type_class = objects::object_type_class(header.type_code)
            .as_str()
            .to_string();
        result.push((
            obj.handle.0,
            obj.offset,
            header.data_size,
            header.type_code,
            type_name,
            type_class,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, type_codes, limit=None))]
pub fn read_object_records_by_type(
    path: &str,
    type_codes: Vec<u16>,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u32, u32, u16, Vec<u8>)>> {
    if type_codes.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let filter: std::collections::HashSet<u16> = type_codes.into_iter().collect();
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let header = objects::parse_object_header_r2000(&bytes, obj.offset).map_err(to_py_err)?;
        if !filter.contains(&header.type_code) {
            continue;
        }
        let (start, end) = header.record_range();
        let record = bytes[start..end].to_vec();
        result.push((
            obj.handle.0,
            obj.offset,
            header.data_size,
            header.type_code,
            record,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_line_entities(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, f64, f64, f64, f64, f64, f64)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x13 {
            continue;
        }
        let entity = entities::decode_line(&mut reader).map_err(to_py_err)?;
        result.push((
            entity.handle,
            entity.start.0,
            entity.start.1,
            entity.start.2,
            entity.end.0,
            entity.end.1,
            entity.end.2,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_arc_entities(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, f64, f64, f64, f64, f64, f64)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x11 {
            continue;
        }
        let entity = entities::decode_arc(&mut reader).map_err(to_py_err)?;
        result.push((
            entity.handle,
            entity.center.0,
            entity.center.1,
            entity.center.2,
            entity.radius,
            entity.angle_start,
            entity.angle_end,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_insert_entities(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, f64, f64, f64, f64, f64, f64, f64)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x07 {
            continue;
        }
        let entity = entities::decode_insert(&mut reader).map_err(to_py_err)?;
        result.push((
            entity.handle,
            entity.position.0,
            entity.position.1,
            entity.position.2,
            entity.scale.0,
            entity.scale.1,
            entity.scale.2,
            entity.rotation,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_polyline_2d_entities(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u16, u16, f64, f64, f64, f64)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x0F {
            continue;
        }
        let entity = entities::decode_polyline_2d(&mut reader).map_err(to_py_err)?;
        result.push((
            entity.handle,
            entity.flags,
            entity.curve_type,
            entity.width_start,
            entity.width_end,
            entity.thickness,
            entity.elevation,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_polyline_2d_entities_interpreted(
    path: &str,
    limit: Option<usize>,
) -> PyResult<
    Vec<(
        u64,
        u16,
        u16,
        String,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
    )>,
> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x0F {
            continue;
        }
        let entity = entities::decode_polyline_2d(&mut reader).map_err(to_py_err)?;
        let info = entity.flags_info;
        let curve_label = entity.curve_type_info.label().to_string();
        result.push((
            entity.handle,
            entity.flags,
            entity.curve_type,
            curve_label,
            info.closed,
            info.curve_fit,
            info.spline_fit,
            info.is_3d_polyline,
            info.is_3d_mesh,
            info.is_closed_mesh,
            info.is_polyface_mesh,
            info.continuous_linetype,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_lwpolyline_entities(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u16, Vec<(f64, f64)>)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x4D {
            continue;
        }
        let entity = entities::decode_lwpolyline(&mut reader).map_err(to_py_err)?;
        result.push((entity.handle, entity.flags, entity.vertices));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_polyline_2d_with_vertices(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u16, Vec<(f64, f64, f64)>)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut sorted = index.objects.clone();
    sorted.sort_by_key(|obj| obj.offset);

    let mut vertex_map = std::collections::HashMap::new();
    for obj in sorted.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code == 0x0A {
            let vertex = entities::decode_vertex_2d(&mut reader).map_err(to_py_err)?;
            vertex_map.insert(vertex.handle, vertex);
        }
    }

    let mut result = Vec::new();
    let mut i = 0usize;
    while i < sorted.len() {
        let obj = sorted[i];
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x0F {
            i += 1;
            continue;
        }
        let poly = entities::decode_polyline_2d(&mut reader).map_err(to_py_err)?;
        let mut vertices: Vec<(f64, f64, f64)> = Vec::new();
        let use_vertex_z = poly.flags_info.is_3d_polyline
            || poly.flags_info.is_3d_mesh
            || poly.flags_info.is_polyface_mesh;

        if !poly.owned_handles.is_empty() {
            for handle in &poly.owned_handles {
                if let Some(vertex) = vertex_map.get(handle) {
                    let z = if use_vertex_z {
                        vertex.position.2
                    } else {
                        poly.elevation
                    };
                    vertices.push((vertex.position.0, vertex.position.1, z));
                }
            }
            i += 1;
        } else {
            let mut j = i + 1;
            while j < sorted.len() {
                let next = sorted[j];
                let next_record =
                    objects::parse_object_record(&bytes, next.offset).map_err(to_py_err)?;
                let mut next_reader = next_record.bit_reader();
                let next_type = next_reader.read_bs().map_err(to_py_err)?;
                if next_type == 0x0A {
                    let vertex = entities::decode_vertex_2d(&mut next_reader).map_err(to_py_err)?;
                    let z = if use_vertex_z {
                        vertex.position.2
                    } else {
                        poly.elevation
                    };
                    vertices.push((vertex.position.0, vertex.position.1, z));
                    j += 1;
                    continue;
                }
                if next_type == 0x06 {
                    let _seqend = entities::decode_seqend(&mut next_reader).map_err(to_py_err)?;
                    j += 1;
                }
                break;
            }
            i = j;
        }

        if poly.flags_info.closed && vertices.len() > 1 {
            let first = vertices[0];
            let last = *vertices.last().unwrap();
            if !points_equal_3d(first, last) {
                vertices.push(first);
            }
        }

        result.push((poly.handle, poly.flags, vertices));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }

    Ok(result)
}

#[pyfunction(signature = (path, segments_per_span=8, limit=None))]
pub fn decode_polyline_2d_with_vertices_interpolated(
    path: &str,
    segments_per_span: usize,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u16, bool, Vec<(f64, f64, f64)>)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut sorted = index.objects.clone();
    sorted.sort_by_key(|obj| obj.offset);

    let mut vertex_map = std::collections::HashMap::new();
    for obj in sorted.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code == 0x0A {
            let vertex = entities::decode_vertex_2d(&mut reader).map_err(to_py_err)?;
            vertex_map.insert(vertex.handle, vertex);
        }
    }

    let mut result = Vec::new();
    let mut i = 0usize;
    while i < sorted.len() {
        let obj = sorted[i];
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x0F {
            i += 1;
            continue;
        }
        let poly = entities::decode_polyline_2d(&mut reader).map_err(to_py_err)?;
        let mut vertices: Vec<(f64, f64, f64)> = Vec::new();
        let use_vertex_z = poly.flags_info.is_3d_polyline
            || poly.flags_info.is_3d_mesh
            || poly.flags_info.is_polyface_mesh;

        if !poly.owned_handles.is_empty() {
            for handle in &poly.owned_handles {
                if let Some(vertex) = vertex_map.get(handle) {
                    let z = if use_vertex_z {
                        vertex.position.2
                    } else {
                        poly.elevation
                    };
                    vertices.push((vertex.position.0, vertex.position.1, z));
                }
            }
            i += 1;
        } else {
            let mut j = i + 1;
            while j < sorted.len() {
                let next = sorted[j];
                let next_record =
                    objects::parse_object_record(&bytes, next.offset).map_err(to_py_err)?;
                let mut next_reader = next_record.bit_reader();
                let next_type = next_reader.read_bs().map_err(to_py_err)?;
                if next_type == 0x0A {
                    let vertex = entities::decode_vertex_2d(&mut next_reader).map_err(to_py_err)?;
                    let z = if use_vertex_z {
                        vertex.position.2
                    } else {
                        poly.elevation
                    };
                    vertices.push((vertex.position.0, vertex.position.1, z));
                    j += 1;
                    continue;
                }
                if next_type == 0x06 {
                    let _seqend = entities::decode_seqend(&mut next_reader).map_err(to_py_err)?;
                    j += 1;
                }
                break;
            }
            i = j;
        }

        let mut applied = false;
        let should_interpolate = poly.flags_info.curve_fit
            || poly.flags_info.spline_fit
            || matches!(
                poly.curve_type_info,
                entities::PolylineCurveType::QuadraticBSpline
                    | entities::PolylineCurveType::CubicBSpline
                    | entities::PolylineCurveType::Bezier
            );

        if should_interpolate && vertices.len() > 1 {
            let base = strip_closure(vertices);
            let interpolated =
                entities::catmull_rom_spline(&base, poly.flags_info.closed, segments_per_span)
                    .map_err(to_py_err)?;
            vertices = interpolated;
            applied = true;
        } else if poly.flags_info.closed && vertices.len() > 1 {
            let first = vertices[0];
            let last = *vertices.last().unwrap();
            if !points_equal_3d(first, last) {
                vertices.push(first);
            }
        }

        result.push((poly.handle, poly.flags, applied, vertices));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }

    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_vertex_2d_entities(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u16, f64, f64, f64, f64, f64, f64, f64)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut result = Vec::new();
    for obj in index.objects.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x0A {
            continue;
        }
        let vertex = entities::decode_vertex_2d(&mut reader).map_err(to_py_err)?;
        result.push((
            vertex.handle,
            vertex.flags,
            vertex.position.0,
            vertex.position.1,
            vertex.position.2,
            vertex.start_width,
            vertex.end_width,
            vertex.bulge,
            vertex.tangent_dir,
        ));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }
    Ok(result)
}

#[pyfunction(signature = (path, limit=None))]
pub fn decode_polyline_2d_with_vertex_data(
    path: &str,
    limit: Option<usize>,
) -> PyResult<Vec<(u64, u16, Vec<(f64, f64, f64, f64, f64, f64, f64, u16)>)>> {
    let bytes = file_open::read_file(path).map_err(to_py_err)?;
    let version = version::detect_version(&bytes[..6.min(bytes.len())]).map_err(to_py_err)?;
    if !version.is_supported() {
        return Err(PyValueError::new_err(format!(
            "unsupported DWG version: {}",
            version.as_str()
        )));
    }
    let index = objects::build_object_index(&bytes, &Default::default()).map_err(to_py_err)?;
    let mut sorted = index.objects.clone();
    sorted.sort_by_key(|obj| obj.offset);

    let mut vertex_map = std::collections::HashMap::new();
    for obj in sorted.iter() {
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code == 0x0A {
            let vertex = entities::decode_vertex_2d(&mut reader).map_err(to_py_err)?;
            vertex_map.insert(vertex.handle, vertex);
        }
    }

    let mut result = Vec::new();
    let mut i = 0usize;
    while i < sorted.len() {
        let obj = sorted[i];
        let record = objects::parse_object_record(&bytes, obj.offset).map_err(to_py_err)?;
        let mut reader = record.bit_reader();
        let type_code = reader.read_bs().map_err(to_py_err)?;
        if type_code != 0x0F {
            i += 1;
            continue;
        }
        let poly = entities::decode_polyline_2d(&mut reader).map_err(to_py_err)?;
        let mut vertices: Vec<(f64, f64, f64, f64, f64, f64, f64, u16)> = Vec::new();
        let use_vertex_z = poly.flags_info.is_3d_polyline
            || poly.flags_info.is_3d_mesh
            || poly.flags_info.is_polyface_mesh;

        if !poly.owned_handles.is_empty() {
            for handle in &poly.owned_handles {
                if let Some(vertex) = vertex_map.get(handle) {
                    let z = if use_vertex_z {
                        vertex.position.2
                    } else {
                        poly.elevation
                    };
                    vertices.push((
                        vertex.position.0,
                        vertex.position.1,
                        z,
                        vertex.start_width,
                        vertex.end_width,
                        vertex.bulge,
                        vertex.tangent_dir,
                        vertex.flags,
                    ));
                }
            }
            i += 1;
        } else {
            let mut j = i + 1;
            while j < sorted.len() {
                let next = sorted[j];
                let next_record =
                    objects::parse_object_record(&bytes, next.offset).map_err(to_py_err)?;
                let mut next_reader = next_record.bit_reader();
                let next_type = next_reader.read_bs().map_err(to_py_err)?;
                if next_type == 0x0A {
                    let vertex = entities::decode_vertex_2d(&mut next_reader).map_err(to_py_err)?;
                    let z = if use_vertex_z {
                        vertex.position.2
                    } else {
                        poly.elevation
                    };
                    vertices.push((
                        vertex.position.0,
                        vertex.position.1,
                        z,
                        vertex.start_width,
                        vertex.end_width,
                        vertex.bulge,
                        vertex.tangent_dir,
                        vertex.flags,
                    ));
                    j += 1;
                    continue;
                }
                if next_type == 0x06 {
                    let _seqend = entities::decode_seqend(&mut next_reader).map_err(to_py_err)?;
                    j += 1;
                }
                break;
            }
            i = j;
        }

        if poly.flags_info.closed && vertices.len() > 1 {
            let first = vertices[0];
            let last = *vertices.last().unwrap();
            if !points_equal_3d_with_data(first, last) {
                vertices.push(first);
            }
        }

        result.push((poly.handle, poly.flags, vertices));
        if let Some(limit) = limit {
            if result.len() >= limit {
                break;
            }
        }
    }

    Ok(result)
}

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(hello_from_bin, module)?)?;
    module.add_function(wrap_pyfunction!(detect_version, module)?)?;
    module.add_function(wrap_pyfunction!(list_section_locators, module)?)?;
    module.add_function(wrap_pyfunction!(read_section_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(list_object_map_entries, module)?)?;
    module.add_function(wrap_pyfunction!(list_object_headers, module)?)?;
    module.add_function(wrap_pyfunction!(list_object_headers_with_type, module)?)?;
    module.add_function(wrap_pyfunction!(list_object_headers_by_type, module)?)?;
    module.add_function(wrap_pyfunction!(read_object_records_by_type, module)?)?;
    module.add_function(wrap_pyfunction!(decode_line_entities, module)?)?;
    module.add_function(wrap_pyfunction!(decode_arc_entities, module)?)?;
    module.add_function(wrap_pyfunction!(decode_insert_entities, module)?)?;
    module.add_function(wrap_pyfunction!(decode_polyline_2d_entities, module)?)?;
    module.add_function(wrap_pyfunction!(
        decode_polyline_2d_entities_interpreted,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(decode_lwpolyline_entities, module)?)?;
    module.add_function(wrap_pyfunction!(decode_polyline_2d_with_vertices, module)?)?;
    module.add_function(wrap_pyfunction!(
        decode_polyline_2d_with_vertices_interpolated,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(decode_vertex_2d_entities, module)?)?;
    module.add_function(wrap_pyfunction!(
        decode_polyline_2d_with_vertex_data,
        module
    )?)?;
    Ok(())
}

fn to_py_err(err: DwgError) -> PyErr {
    let message = err.to_string();
    match err.kind {
        ErrorKind::Io => PyIOError::new_err(message),
        ErrorKind::Format | ErrorKind::Decode | ErrorKind::Resolve | ErrorKind::Unsupported => {
            PyValueError::new_err(message)
        }
        ErrorKind::NotImplemented => PyNotImplementedError::new_err(message),
    }
}

fn points_equal_3d(a: (f64, f64, f64), b: (f64, f64, f64)) -> bool {
    const EPS: f64 = 1e-9;
    (a.0 - b.0).abs() < EPS && (a.1 - b.1).abs() < EPS && (a.2 - b.2).abs() < EPS
}

fn strip_closure(mut points: Vec<(f64, f64, f64)>) -> Vec<(f64, f64, f64)> {
    if points.len() > 1 {
        let first = points[0];
        let last = *points.last().unwrap();
        if points_equal_3d(first, last) {
            points.pop();
        }
    }
    points
}

fn points_equal_3d_with_data(
    a: (f64, f64, f64, f64, f64, f64, f64, u16),
    b: (f64, f64, f64, f64, f64, f64, f64, u16),
) -> bool {
    points_equal_3d((a.0, a.1, a.2), (b.0, b.1, b.2))
}
